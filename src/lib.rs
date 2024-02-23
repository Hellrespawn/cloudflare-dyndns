// #![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(unknown_lints)] // For nightly lints
#![allow(clippy::uninlined_format_args)]

pub mod cli;
pub mod cloudflare;
pub mod config;
pub mod fs;
pub mod ip;
pub mod network;

use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const LOG_KEY: &str = "LOG";

/// Install `color_eyre` and enable tracing. Defaults to `Level::INFO`.
pub fn init() -> color_eyre::Result<()> {
    color_eyre::install()?;

    if (std::env::var_os(LOG_KEY)).is_none() {
        std::env::set_var(LOG_KEY, "info");
    }

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("LOG"))
        .init();

    Ok(())
}
