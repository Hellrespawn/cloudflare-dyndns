// #![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(unknown_lints)] // For nightly lints
#![allow(clippy::uninlined_format_args)]

mod args;
mod config;

pub mod cli;
pub mod cloudflare;
pub mod ip;

pub use args::Args;
pub use config::Config;

use fs_err as fs;
use once_cell::sync::Lazy;
use std::path::{Path, PathBuf};

pub const API_URL: &str = "https://api.cloudflare.com/client/v4";

static SYSTEM_CONFIG_DIRECTORY: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from("/etc/cloudflare-dyndns"));

static USER_CONFIG_DIRECTORY: Lazy<PathBuf> = Lazy::new(|| {
    dirs::config_dir().expect("Unable to read user config directory.")
});

const CONFIG_FILE_NAME: &str = "cloudflare-dyndns.conf";
const PREVIOUS_IP_FILE_NAME: &str = "last_ip";

pub static SYSTEM_CONFIG_PATH: Lazy<PathBuf> =
    Lazy::new(|| SYSTEM_CONFIG_DIRECTORY.join(CONFIG_FILE_NAME));

pub static SYSTEM_PREVIOUS_IP_PATH: Lazy<PathBuf> =
    Lazy::new(|| SYSTEM_CONFIG_DIRECTORY.join(PREVIOUS_IP_FILE_NAME));

pub static USER_CONFIG_PATH: Lazy<PathBuf> =
    Lazy::new(|| USER_CONFIG_DIRECTORY.join(CONFIG_FILE_NAME));

pub static USER_PREVIOUS_IP_PATH: Lazy<PathBuf> =
    Lazy::new(|| USER_CONFIG_DIRECTORY.join(PREVIOUS_IP_FILE_NAME));

pub fn read_file_optional(path: &Path) -> Option<String> {
    fs::read_to_string(path).ok()
}
