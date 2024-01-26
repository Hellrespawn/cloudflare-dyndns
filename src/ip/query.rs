use std::net::Ipv4Addr;

use async_trait::async_trait;
use color_eyre::Result;

use crate::Args;

#[async_trait]
pub trait IpQuery {
    async fn get_public_ip_address(&self) -> Result<Ipv4Addr>;
}

pub struct IpEcho;

#[async_trait]
impl IpQuery for IpEcho {
    async fn get_public_ip_address(&self) -> Result<Ipv4Addr> {
        let ip = reqwest::get("https://ipecho.net/plain").await?.text().await?;

        Ok(ip.parse()?)
    }
}

pub fn get_ip_query_from_args(_args: &Args) -> &dyn IpQuery {
    &IpEcho
}
