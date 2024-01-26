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
pub mod config;

pub mod cloudflare;
pub mod ip;

pub use args::Args;
use camino::Utf8Path;
pub use config::Settings;
use fs_err as fs;

pub fn read_file_optional(path: &Utf8Path) -> Option<String> {
    fs::read_to_string(path).ok().map(|s| s.trim().to_owned())
}

use color_eyre::eyre::eyre;
use color_eyre::Result;
use reqwest::header::HeaderMap;
use reqwest::Client;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

/// Create client with Content-Type and Authorization headers.
pub fn create_reqwest_client(token: &str) -> Result<Client> {
    let mut headers = HeaderMap::new();

    headers.insert(
        "Content-Type",
        "application/json"
            .parse()
            .map_err(|_| eyre!("Invalid 'Content-Type' header."))?,
    );
    headers.insert(
        "Authorization",
        format!("Bearer {}", token)
            .parse()
            .map_err(|_| eyre!("Invalid 'Authorization' header."))?,
    );

    let client = Client::builder().default_headers(headers).build()?;

    Ok(client)
}
