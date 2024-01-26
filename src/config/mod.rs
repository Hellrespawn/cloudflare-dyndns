use std::collections::HashMap;

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::eyre;
use color_eyre::Result;

mod paths;
mod settings;
mod zone;

use paths::CONFIG_PATHS;
pub use settings::Settings;
pub use zone::ZoneIdentifier;

#[derive(Debug, Default)]
struct SettingsDTO {
    config_path: Utf8PathBuf,
    token: Option<String>,
    zone_id: Option<String>,
    zone_name: Option<String>,
}

impl SettingsDTO {
    fn from_file(path: &Utf8Path) -> Result<Option<Self>> {
        if path.is_file() {
            let mut map = dotenvy::from_path_iter(path)?
                .map(|pair| Ok(pair?))
                .collect::<Result<HashMap<String, String>>>()?;

            let token = map.remove("CLOUDFLARE_TOKEN");
            let zone_id = map.remove("CLOUDFLARE_ZONE_ID");
            let zone_name = map.remove("CLOUDFLARE_ZONE_NAME");

            Ok(Some(Self {
                token,
                zone_id,
                zone_name,
                config_path: path
                    .parent()
                    .expect("File path always has parent")
                    .to_owned(),
            }))
        } else if path.exists() {
            Err(eyre!("Path exists but is not a file: {}", path))
        } else {
            Ok(None)
        }
    }
}

fn create_missing_options_error(missing: &[&str]) -> color_eyre::Report {
    eyre!(
        "Unable to read variables: {}\nCheck your configuration files:\n{}",
        missing.join(", "),
        *CONFIG_PATHS
    )
}
