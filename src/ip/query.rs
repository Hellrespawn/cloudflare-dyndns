use color_eyre::Result;
use std::net::Ipv4Addr;

pub trait IpQuery {
    fn get_public_ip_address(
        &self,
    ) -> impl std::future::Future<Output = Result<Ipv4Addr>> + Send;
}

pub struct IpEcho;

impl IpQuery for IpEcho {
    async fn get_public_ip_address(&self) -> Result<Ipv4Addr> {
        let ip = reqwest::get("https://ipecho.net/plain").await?.text().await?;

        Ok(ip.parse()?)
    }
}
