use camino::Utf8PathBuf;
use color_eyre::eyre::eyre;
use color_eyre::Result;

use super::args::Args;
use super::paths::CONFIG_PATHS;
use crate::config::env::Env;
use crate::config::ip::IpAddress;
use crate::config::zone::ZoneIdentifier;
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

    pub fn default_from_args(args: Args) -> Result<Self> {
        let env = if let Some(config_file) = &args.config_file {
            let env = Env::from_file(config_file)?.ok_or(eyre!(
                "Unable to read config from file: {}",
                config_file
            ))?;

            Some(env)
        } else {
            let system_env = Env::from_file(&CONFIG_PATHS.system_config())?;

            let user_env = Env::from_file(&CONFIG_PATHS.user_config())?;

            match (system_env, user_env) {
                (None, None) => None,
                (None, Some(env)) | (Some(env), None) => Some(env),
                (Some(system_env), Some(user_env)) => {
                    Some(system_env.merge(user_env))
                },
            }
        };

        let settings = if let Some(env) = env {
            Settings::from_args_and_env(args, env)
        } else {
            Settings::from_args(args, CONFIG_PATHS.user_config())
        }?;

        Ok(settings)
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
