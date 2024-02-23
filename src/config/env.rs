use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use tracing::{error, trace};

pub struct Env {
    pub ip_address: Option<String>,
    pub ip_url: Option<String>,
    pub token: Option<String>,
    pub zone_id: Option<String>,
    pub zone_name: Option<String>,

    pub path: Utf8PathBuf,
}

impl Env {
    pub fn from_file(path: &Utf8Path) -> Result<Option<Self>> {
        if path.is_file() {
            let mut map = dotenvy::from_path_iter(path)?
                .map(|pair| Ok(pair?))
                .collect::<Result<HashMap<String, String>>>()?;

            let env = Self {
                ip_address: map.remove("IP_ADDRESS"),
                ip_url: map.remove("IP_URL"),

                token: map.remove("CLOUDFLARE_TOKEN"),
                zone_id: map.remove("CLOUDFLARE_ZONE_ID"),
                zone_name: map.remove("CLOUDFLARE_ZONE_NAME"),
                path: path.to_owned(),
            };

            if map.is_empty() {
                trace!("Read configuration from {}", path);

                Ok(Some(env))
            } else {
                for (key, value) in map {
                    error!("Unknown option: '{}'='{}'", key, value);
                }

                Err(eyre!("Found unknown keys in configuration."))
            }
        } else if path.exists() {
            Err(eyre!("Path exists but is not a file: {}", path))
        } else {
            Ok(None)
        }
    }
}
