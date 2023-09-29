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
pub use config::{Settings, CONFIG_PATHS};

use fs_err as fs;
use camino::Utf8Path;

pub fn read_file_optional(path: &Utf8Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_owned())
}
