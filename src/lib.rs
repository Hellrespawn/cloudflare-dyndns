// #![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(unknown_lints)] // For nightly lints
#![allow(clippy::uninlined_format_args)]

pub mod cli;
pub mod cloudflare_api;
pub mod config;
pub mod public_ip;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use reqwest::header::HeaderMap;
use reqwest::Client;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");

const LOG_KEY: &str = "LOG";

/// Install `color_eyre` and enable tracing. Defaults to `Level::INFO`.
pub fn init() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let filter = EnvFilter::builder().with_env_var(LOG_KEY).from_env()?;

    tracing_subscriber::registry()
        .with(fmt::layer().compact())
        .with(filter)
        .init();

    Ok(())
}

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

    let client =
        Client::builder().default_headers(headers).use_rustls_tls().build()?;

    Ok(client)
}
