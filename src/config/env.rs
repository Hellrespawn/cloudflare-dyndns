use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::eyre;
use color_eyre::Result;

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

            Ok(Some(Self {
                ip_address: map.remove("IP_ADDRESS"),
                ip_url: map.remove("IP_URL"),

                token: map.remove("CLOUDFLARE_TOKEN"),
                zone_id: map.remove("CLOUDFLARE_ZONE_ID"),
                zone_name: map.remove("CLOUDFLARE_ZONE_NAME"),
                path: path.to_owned(),
            }))
        } else if path.exists() {
            Err(eyre!("Path exists but is not a file: {}", path))
        } else {
            Ok(None)
        }
    }

    #[must_use]
    pub fn merge(mut self, other: Self) -> Self {
        self.ip_address = other.ip_address.or(self.ip_address);
        self.ip_url = other.ip_url.or(self.ip_url);
        self.token = other.token.or(self.token);
        self.zone_id = other.zone_id.or(self.zone_id);
        self.zone_name = other.zone_name.or(self.zone_name);

        self
    }
}