use deltakit::gamedata::parse_filename;
use deltakit::init;
use deltakit::savefile::SaveData;
use log::{error, info};
use std::env::args;
use std::process;
use deltakit::merging::{merge_savefiles, MergeResult};

/// Git should call this as a merge driver.
///
/// ```
/// [merge "deltamerge"]
/// name = deltamerge
/// driver = deltamerge %O %A %B %L %P
/// ```
fn main() {
    init();
    let args = args().collect::<Vec<_>>();

    if args.len() < 6 {
        error!("Usage: deltamerge <common_ancestor> <ours> <theirs> <merge_marker> <output_name>");
        process::exit(255);
    }

    // Path to the common base file before any changes
    let common_ancestor = &args[1];
    // Path to our version of the file
    let ours = &args[2];
    // Path to their version of the file
    let theirs = &args[3];
    // Length of merge markers
    let merge_marker_len = &args[4];
    // Path where the merged result will be saved
    let output_name = &args[5];

    let (chapter_id, save_id) = parse_filename(output_name);

    // Read current file lines
    let ours_str = std::fs::read_to_string(ours).unwrap();
    let ours_lines = ours_str.lines().collect::<Vec<_>>();
    let ours_data = SaveData::read(chapter_id, &ours_lines).unwrap();

    let theirs_str = std::fs::read_to_string(&theirs).unwrap();
    let theirs_lines = theirs_str.lines().collect::<Vec<_>>();
    let theirs_data = SaveData::read(chapter_id, &theirs_lines).unwrap();

    let ancestor_str = std::fs::read_to_string(&common_ancestor).unwrap();
    let ancestor_lines = ancestor_str.lines().collect::<Vec<_>>();
    let ancestor_data = if ancestor_lines.len() > 0 {
        Some(SaveData::read(chapter_id, &ancestor_lines).unwrap())
    } else { None };

    info!("Detected save {save_id} of chapter {chapter_id}.");

    let merge_result = merge_savefiles(
        &ours_data,
        &theirs_data,
        ancestor_data.as_ref(),
    ).expect("failed to merge save files");

    println!("{:?}", merge_result);

    // Check if there are any conflicts in the merge result
    let has_conflicts = merge_result.iter().any(|result| {
        matches!(result, MergeResult::Conflict { .. })
    });

    // Convert each MergeResult to a string using to_merge_string and join with \r\n
    let merge_strings: Vec<String> = merge_result.iter()
        .map(|result| result.to_merge_string(merge_marker_len.parse().unwrap_or(7)))
        .collect();
    let merged_content = merge_strings.join("\r\n");

    // Write the result back to the ours file
    std::fs::write(ours, merged_content).expect("Failed to write merge result to file");

    info!("Successfully wrote merge result to {}", ours);

    if has_conflicts {
        info!("Merge conflicts detected. Exiting with code 1 to notify git.");
        process::exit(1);
    } else {
        process::exit(0);
    }
}
