extern crate core;

pub mod gamedata;
pub mod savefile;
pub mod iter;

use log::{debug, LevelFilter};

pub fn message() -> &'static str {
    "Hello deltakit!"
}

pub fn init() {
    pretty_env_logger::formatted_builder().filter_level(LevelFilter::Info).parse_default_env()
        .init();
    // pretty_env_logger::init();
    debug!("deltakit initialized.");
}
