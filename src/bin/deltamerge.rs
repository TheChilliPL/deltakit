use std::env::args;
use std::{iter, process};
use std::fs::metadata;
use log::{info, error};
use deltakit::init;
use regex::Regex;
use deltakit::gamedata::parse_filename;
use deltakit::savefile::SaveMetadata;

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
    let merge_marker = &args[4];
    // Path where the merged result will be saved
    let output_name = &args[5];

    let (chapter_id, save_id) = parse_filename(output_name);
    
    // Read current file lines
    let ours_str = std::fs::read_to_string(ours).unwrap();
    let ours_lines = ours_str.lines().collect::<Vec<_>>();
    let metadata = SaveMetadata::read(chapter_id, &ours_lines);
    
    info!("Detected save {save_id} of chapter {chapter_id}.");
    
    info!("{}", metadata.display_info());

    std::thread::sleep(std::time::Duration::from_secs(10));
    process::exit(255);
}