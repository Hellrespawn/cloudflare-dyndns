pub mod ip_cache;

use std::net::Ipv4Addr;

use color_eyre::Result;

pub async fn get_public_ip_address(url: &str) -> Result<Ipv4Addr> {
    Ok(reqwest::get(url).await?.text().await?.parse()?)
}
