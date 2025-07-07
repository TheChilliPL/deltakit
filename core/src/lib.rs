extern crate core;

pub mod gamedata;
pub mod iter;
mod save_parser;
pub mod savefile;
pub mod merging;

use log::{LevelFilter, debug, error};
use std::{panic, process};

pub fn message() -> &'static str {
    "Hello deltakit!"
}

pub fn init() {
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .parse_default_env()
        .init();
    // pretty_env_logger::init();

    // Ensure the program returns 255 on panic so that Git doesn't interpret it as a merge conflict.
    panic::set_hook(Box::new(|info| {
        error!("deltakit panicked! {}", info);
        process::exit(255);
    }));

    debug!("deltakit initialized.");
}
