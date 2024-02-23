// #![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
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

const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");
const LOG_KEY: &str = "LOG";

/// Install `color_eyre` and enable tracing. Defaults to `Level::INFO`.
pub fn init() -> color_eyre::Result<()> {
    color_eyre::install()?;

    if (std::env::var(LOG_KEY)).is_err() {
        std::env::set_var(LOG_KEY, format!("none,{CRATE_NAME}=info"));
    } else {
        let env_var =
            format!("none,{CRATE_NAME}={}", std::env::var(LOG_KEY).unwrap());

        dbg!(&env_var);

        std::env::set_var(LOG_KEY, env_var);
    }

    let filter = EnvFilter::builder()
        .with_env_var(LOG_KEY)
        .with_default_directive((format!("{CRATE_NAME}=info")).parse()?)
        .from_env()?;

    tracing_subscriber::registry()
        .with(fmt::layer().compact())
        .with(filter)
        .init();

    Ok(())
}
