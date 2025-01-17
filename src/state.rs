use std::net::Ipv4Addr;

use camino::Utf8PathBuf;
use derive_builder::Builder;
use reqwest::Client;

use crate::ip_cache::IpCache;

#[derive(Debug, Builder)]
pub struct ApplicationState {
    pub client: Client,
    pub config_path: Utf8PathBuf,
    pub ip_cache: IpCache,
    pub ip_cache_path: Utf8PathBuf,
    pub public_ip_address: Ipv4Addr,
    pub preview: bool,
    pub force: bool,
}
