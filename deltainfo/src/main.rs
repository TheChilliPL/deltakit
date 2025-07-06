use deltakit::gamedata::parse_filename;
use deltakit::savefile::SaveData;
use deltakit::init;
use log::{error, info};
use std::env::args;
use std::process;
use clap::Parser;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// Save file to parse
    file: String,
    /// Chapter number. If not specified, will try to parse from filename.
    #[arg(short, long)]
    chapter: Option<i32>,
    /// Print debug save info instead of basic information.
    #[arg(short, long)]
    debug: bool,
}

fn main() {
    init();
    
    let cli = Args::parse();

    let path = &cli.file;
    let file_content = std::fs::read_to_string(path).unwrap();
    let file_lines = file_content.lines().collect::<Vec<_>>();

    let chapter_id = cli.chapter.unwrap_or_else(|| parse_filename(path).0);
    let metadata = SaveData::read(chapter_id, &file_lines).unwrap();
    
    if cli.debug {
        info!("{:#?}", metadata);
    } else {
        info!("{}", metadata.display_info());
    }
}
