use clap::Parser;
use deltakit::gamedata::parse_filename;
use deltakit::gamedata::rooms::try_get_room_name;
use deltakit::iter::{IterExt, SingleError};
use deltakit::savefile::SaveData;
use deltakit::init;
use log::{error, info};
use std::process::Command;
use std::{process, str};

#[derive(Parser, Debug)]
#[command()]
struct Args {
    /// Commit message to use.
    ///
    /// If multiple are provided, there's an empty line after the first one, and single
    /// line breaks between the rest.
    #[arg(short, long)]
    message: Vec<String>,
    /// Room name to use in the message.
    /// If not provided, the room name extracted from the game is used (supported up to chapter 4).
    #[arg(short, long)]
    room: Option<String>,
}

fn main() {
    init();

    let cli = Args::parse();

    // Use git status to determine which save file was changed
    let status = Command::new("git").arg("status").output();

    if status.is_err() || !status.as_ref().unwrap().status.success() {
        error!("Failed to run git status");
        process::exit(255);
    }

    let status = str::from_utf8(&status.as_ref().unwrap().stdout);

    if status.is_err() {
        error!("Failed to parse git status output");
        process::exit(255);
    }

    let status = status.unwrap();

    let status = status
        .lines()
        .map(|line| line.trim())
        .filter(|line| line.starts_with("modified:") | line.starts_with("new file:"))
        .flat_map(|line| {
            let filename = line.split(":").nth(1).unwrap_or("").trim();
            let (chapter, save) = parse_filename(filename);
            if chapter > 0 {
                Some((filename, chapter, save))
            } else {
                None
            }
        })
        .expect_single();

    if let Err(err) = status {
        match err {
            SingleError::None => {
                error!("No save file modified");
                process::exit(255);
            }
            SingleError::Multiple => {
                error!("Multiple save files modified");
                process::exit(255);
            }
        }
    }

    let (filename, chapter, _save) = status.unwrap();

    let save_data = std::fs::read_to_string(filename).unwrap();
    let save_lines = save_data.lines().collect::<Vec<_>>();

    let save_info = SaveData::read(chapter, &save_lines).unwrap();

    let room = if let Some(ref room) = cli.room { room } else {
        let room_id = save_info.room_id;
        let room_name = try_get_room_name(room_id / 10000, room_id % 10000);

        if room_name.is_none() {
            error!("Failed to find room name for room ID {}", room_id);
            info!("Hint: You can use -r|--room to specify the room name manually");
            process::exit(255);
        }

        room_name.unwrap()
    };

    let time_played_secs = save_info.time_played.as_secs();
    let time_played_h = time_played_secs / 3600;
    let time_played_m = (time_played_secs % 3600) / 60;
    let time_played_s = time_played_secs % 60;
    let time_played = format!(
        "{}h{:02}m{:02}s",
        time_played_h, time_played_m, time_played_s
    );

    let mut custom_message = cli.message;
    if !custom_message.is_empty() {
        custom_message[0] = custom_message[0].clone() + "\n";
        custom_message = custom_message.into_iter().map(|msg| msg + "\n").collect();
    }
    custom_message.push(format!("\"{}\" - {}", room, time_played));

    let commit_message = custom_message.join("");

    let add_result = Command::new("git").arg("add").arg(".").status();

    if add_result.is_err() || !add_result.unwrap().success() {
        error!("Failed to run git add");
        process::exit(255);
    }

    let commit_result = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(&commit_message)
        .output();

    if commit_result.is_err() || !commit_result.as_ref().unwrap().status.success() {
        error!("Failed to run git commit. {}", commit_message);
        process::exit(255);
    }

    info!(
        "Committed successfully. Room {}, played for {}.",
        room, time_played
    );
}
