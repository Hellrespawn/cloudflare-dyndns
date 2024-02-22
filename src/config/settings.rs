use std::net::Ipv4Addr;

use camino::Utf8PathBuf;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use reqwest::Client;

use super::args::Args;
use super::env::Env;
use crate::cloudflare::ListZonesResponse;
use crate::PKG_NAME;

#[derive(Debug)]
pub struct Settings {
    pub ip: IpAddress,

    pub token: String,
    pub zone: ZoneIdentifier,

    pub path: Utf8PathBuf,

    pub force: bool,
    pub preview: bool,
}

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

#[derive(Debug)]
pub enum ZoneIdentifier {
    Id(String),
    Name(String),
    Both { id: String, name: String },
}

impl ZoneIdentifier {
    pub fn new(id: Option<String>, name: Option<String>) -> Result<Self> {
        match (id, name) {
            (None, None) => {
                Err(eyre!("Neither Cloudflare zone ID nor zone name are set."))
            },
            (Some(id), None) => Ok(ZoneIdentifier::Id(id)),
            (None, Some(name)) => Ok(ZoneIdentifier::Name(name)),
            (Some(id), Some(name)) => Ok(ZoneIdentifier::Both { id, name }),
        }
    }

    pub async fn get_zone_id(&self, client: &Client) -> Result<String> {
        match self {
            ZoneIdentifier::Id(id) | ZoneIdentifier::Both { id, .. } => {
                Ok(id.clone())
            },
            ZoneIdentifier::Name(name) => {
                let list_zones_response =
                    ListZonesResponse::get(client).await?;

                let zone_response = list_zones_response
                    .find_by_name(name)
                    .ok_or(eyre!("Unable to find zone with name '{}'", name))?;

                println!("Updating zone '{}'", name);

                Ok(zone_response.id().to_owned())
            },
        }
    }
}

#[allow(clippy::too_many_arguments)]
impl Settings {
    pub fn new(
        token: String,
        ip_address: Option<String>,
        ip_url: Option<String>,
        zone_id: Option<String>,
        zone_name: Option<String>,
        path: Utf8PathBuf,
        force: bool,
        preview: bool,
    ) -> Result<Self> {
        let ip = IpAddress::new(ip_address, ip_url)?;
        let zone = ZoneIdentifier::new(zone_id, zone_name)?;

        Ok(Self { ip, token, zone, path, force, preview })
    }

    pub fn from_args_and_env(args: Args, env: Env) -> Result<Self> {
        let token = args
            .token
            .or(env.token)
            .ok_or(eyre!("Cloudflare token  was not set."))?;

        let ip_address = args.ip_address.address.or(env.ip_address);

        let ip_url = args.ip_address.url.or(env.ip_url);

        let zone_id = args.cloudflare_zone.id.or(env.zone_id);

        let zone_name = args.cloudflare_zone.name.or(env.zone_name);

        Self::new(
            token,
            ip_address,
            ip_url,
            zone_id,
            zone_name,
            env.path,
            args.force,
            args.preview,
        )
    }

    pub fn from_args(args: Args, path: Utf8PathBuf) -> Result<Self> {
        let token =
            args.token.ok_or(eyre!("Cloudflare token  was not set."))?;

        Self::new(
            token,
            args.ip_address.address,
            args.ip_address.url,
            args.cloudflare_zone.id,
            args.cloudflare_zone.name,
            path,
            args.force,
            args.preview,
        )
    }

    pub fn get_config_dir(&self) -> Utf8PathBuf {
        self.path
            .parent()
            .expect("Settings::path should always point to a file")
            .to_owned()
    }

    pub fn get_previous_ip_file(&self) -> Utf8PathBuf {
        self.get_config_dir().join(format!("{PKG_NAME}-previous_ip"))
    }
}
