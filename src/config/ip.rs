use std::net::Ipv4Addr;

use color_eyre::eyre::eyre;
use color_eyre::Result;

#[derive(Debug)]
pub enum IpAddress {
    Address(Ipv4Addr),
    Url(String),
    Both { address: Ipv4Addr, url: String },
}

impl IpAddress {
    pub fn new(
        ip_address: Option<String>,
        ip_url: Option<String>,
    ) -> Result<Self> {
        match (ip_address, ip_url) {
            (None, None) => {
                Err(eyre!("Neither IP address nor IP url are set."))
            },
            (Some(address), None) => Ok(IpAddress::Address(address.parse()?)),
            (None, Some(url)) => Ok(IpAddress::Url(url)),
            (Some(address), Some(url)) => {
                Ok(IpAddress::Both { address: address.parse()?, url })
            },
        }
    }

    pub async fn ip_address(&self) -> Result<Ipv4Addr> {
        match self {
            IpAddress::Url(url) => {
                let ip = reqwest::get(url).await?.text().await?;

                Ok(ip.parse()?)
            },
            IpAddress::Both { address, .. } | IpAddress::Address(address) => {
                Ok(*address)
            },
        }
    }
}
