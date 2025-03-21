// #![warn(missing_docs)]
#![warn(clippy::pedantic)]
//#![warn(clippy::cargo)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(unknown_lints)] // For nightly lints

pub mod cli;
pub mod cloudflare_api;
pub mod config;
pub mod ip_cache;
pub mod state;

use std::net::Ipv4Addr;
use std::str::FromStr;

use color_eyre::Result;
use color_eyre::eyre::eyre;
use reqwest::Client;
use reqwest::header::HeaderMap;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::filter::FilterFn;
use tracing_subscriber::fmt;
use tracing_subscriber::prelude::*;

pub const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const CRATE_NAME: &str = env!("CARGO_CRATE_NAME");

const LOG_KEY: &str = "CFDD_LOG";

/// Install `color_eyre` and enable tracing. Defaults to `Level::INFO`.
pub fn init() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let var = std::env::var_os(LOG_KEY).map(|os| {
        os.into_string().expect("Environment variable is not UTF-8!")
    });

    let level_filter =
        var.map_or(Ok(LevelFilter::OFF), |l| LevelFilter::from_str(&l))?;

    let user_layer = fmt::layer()
        .compact()
        .without_time()
        .with_target(false)
        .with_level(false)
        .with_filter(LevelFilter::INFO)
        .with_filter(FilterFn::new(|m| m.target().starts_with(CRATE_NAME)));

    let dev_layer = fmt::layer()
        .compact()
        .with_filter(level_filter)
        .with_filter(FilterFn::new(|m| m.target().starts_with(CRATE_NAME)));

    tracing_subscriber::registry().with(user_layer).with(dev_layer).init();

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
        format!("Bearer {token}")
            .parse()
            .map_err(|_| eyre!("Invalid 'Authorization' header."))?,
    );

    let client =
        Client::builder().default_headers(headers).use_rustls_tls().build()?;

    Ok(client)
}

pub async fn get_public_ip_address(url: &str) -> Result<Ipv4Addr> {
    Ok(reqwest::get(url).await?.text().await?.parse()?)
}
