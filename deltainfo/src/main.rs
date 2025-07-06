use deltakit::gamedata::parse_filename;
use deltakit::savefile::SaveData;
use deltakit::init;
use log::{error, info};
use std::env::args;
use std::process;

fn main() {
    init();
    let args = args().collect::<Vec<_>>();

    if args.len() < 2 {
        error!("Usage: deltainfo <file>");
        process::exit(255);
    }

    let path = &args[1];
    let file_content = std::fs::read_to_string(path).unwrap();
    let file_lines = file_content.lines().collect::<Vec<_>>();

    let (chapter_id, _) = parse_filename(path);
    let metadata = SaveData::read(chapter_id, &file_lines).unwrap();
    info!("{}", metadata.display_info());
}
