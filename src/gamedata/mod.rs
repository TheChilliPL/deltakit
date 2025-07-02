use regex::Regex;

pub mod rooms;
pub mod spells;
pub mod weapons;
pub mod armors;
pub mod items;
pub mod key_items;
pub mod lightworld_items;
pub mod phone_numbers;
pub mod party_members;

pub fn parse_filename(path: &str) -> (u32, u32) {
    let file_regex = Regex::new(r"^filech(\d)_(\d)$").unwrap();
    let filename = path.split('/').last().unwrap_or(path);

    match file_regex.captures(filename) {
        Some(captures) => {
            let chapter = captures[1].parse::<u32>().unwrap_or(0);
            let save = captures[2].parse::<u32>().unwrap_or(0);
            (chapter, save)
        }
        None => (0, 0)
    }
}