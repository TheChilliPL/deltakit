extern crate core;

pub mod gamedata;
pub mod savefile;

use log::debug;

pub fn message() -> &'static str {
    "Hello deltakit!"
}

pub fn init() {
    pretty_env_logger::init();
    debug!("deltakit initialized.");
}
