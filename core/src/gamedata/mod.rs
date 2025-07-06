use regex::Regex;

pub mod armors;
pub mod items;
pub mod key_items;
pub mod lightworld_items;
pub mod party_members;
pub mod phone_numbers;
pub mod rooms;
pub mod spells;
pub mod weapons;

pub fn parse_filename(path: &str) -> (i32, i32) {
    let file_regex = Regex::new(r"^filech(\d)_(\d)$").unwrap();
    let filename = path.split('/').next_back().unwrap_or(path);

    match file_regex.captures(filename) {
        Some(captures) => {
            let chapter = captures[1].parse::<i32>().unwrap_or(0);
            let save = captures[2].parse::<i32>().unwrap_or(0);
            (chapter, save)
        }
        None => (0, 0),
    }
}
