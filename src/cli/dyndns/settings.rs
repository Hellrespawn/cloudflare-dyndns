use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use nix::unistd::geteuid;

use super::args::Args;
use crate::config::env::Env;
use crate::config::ip::IpAddress;
use crate::config::zone::ZoneIdentifier;
use crate::PKG_NAME;

#[derive(Debug)]
pub struct Settings {
    pub ip: IpAddress,

    pub token: String,
    pub zone: ZoneIdentifier,

    pub config_dir: Utf8PathBuf,

    pub force: bool,
    pub preview: bool,
}

#[allow(clippy::too_many_arguments)]
impl Settings {
    pub fn new(
        token: String,
        ip_address: Option<String>,
        ip_url: Option<String>,
        zone_id: Option<String>,
        zone_name: Option<String>,
        config_dir: Utf8PathBuf,
        force: bool,
        preview: bool,
    ) -> Result<Self> {
        let ip = IpAddress::new(ip_address, ip_url)?;
        let zone = ZoneIdentifier::new(zone_id, zone_name)?;

        Ok(Self { ip, token, zone, config_dir, force, preview })
    }

    pub fn default_from_args(args: Args) -> Result<Self> {
        let env = if let Some(config_file) = &args.config_file {
            let env = Env::from_file(config_file)?.ok_or(eyre!(
                "Unable to read config from file: {}",
                config_file
            ))?;

            Some(env)
        } else {
            None
        };

        let settings = if let Some(env) = env {
            Settings::from_args_and_env(args, env)
        } else {
            Settings::from_args(args, Self::default_config_dir()?)
        }?;

        Ok(settings)
    }

    fn from_args_and_env(args: Args, env: Env) -> Result<Self> {
        let token = args
            .token
            .or(env.token)
            .ok_or(eyre!("Cloudflare token  was not set."))?;

        let ip_address = args.ip_address.address.or(env.ip_address);

        let ip_url = args.ip_address.url.or(env.ip_url);

        let zone_id = args.cloudflare_zone.id.or(env.zone_id);

        let zone_name = args.cloudflare_zone.name.or(env.zone_name);

        let config_dir = env
            .path
            .parent()
            .expect("File should have parent directory.")
            .to_owned();

        Self::new(
            token,
            ip_address,
            ip_url,
            zone_id,
            zone_name,
            config_dir,
            args.force,
            args.preview,
        )
    }

    fn from_args(args: Args, path: Utf8PathBuf) -> Result<Self> {
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

    pub fn config_dir(&self) -> &Utf8Path {
        &self.config_dir
    }

    pub fn previous_ip_file(&self) -> Utf8PathBuf {
        self.config_dir().join(format!("{PKG_NAME}-previous_ip"))
    }

    fn default_config_dir() -> Result<Utf8PathBuf> {
        let mut config_dir: Utf8PathBuf = dirs::config_dir()
            .ok_or(eyre!(
                "Unable to determine config directory for current user."
            ))?
            .try_into()?;

        #[cfg(unix)]
        if geteuid().is_root() {
            config_dir = Utf8PathBuf::from(format!("/etc/{PKG_NAME}"));
        }

        Ok(config_dir)
    }
}
