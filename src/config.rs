use color_eyre::eyre::eyre;
use color_eyre::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::read_file_optional;

#[derive(Debug)]
pub struct ConfigPaths {
    pub system: ConfigPath,
    pub user: ConfigPath,
}

#[derive(Debug)]
pub struct ConfigPath {
    pub settings: PathBuf,
    pub previous_ip: PathBuf,
}

pub static CONFIG_PATHS: Lazy<ConfigPaths> = Lazy::new(|| {
    let pkg_name: &str = env!("CARGO_PKG_NAME");

    let settings_file_name = format!("{pkg_name}.conf");
    let previous_ip_file_name = format!("{pkg_name}-previous_ip");

    let system_config_directory = PathBuf::from(format!("/etc/{pkg_name}"));

    let user_config_directory = dirs::config_dir()
        .expect("Unable to read user config directory.")
        .join(pkg_name);

    ConfigPaths {
        system: ConfigPath {
            settings: system_config_directory.join(&settings_file_name),
            previous_ip: system_config_directory.join(&previous_ip_file_name),
        },
        user: ConfigPath {
            settings: user_config_directory.join(settings_file_name),
            previous_ip: user_config_directory.join(previous_ip_file_name),
        },
    }
});

#[derive(Debug)]
pub struct Settings {
    pub token: String,
    pub zone_id: String,
}

impl Settings {
    pub fn read() -> Result<Settings> {
        let system_settings_dto =
            SettingsDTO::from_file_optional(&CONFIG_PATHS.system.settings)?;

        let settings_dto = system_settings_dto.merge(
            SettingsDTO::from_file_optional(&CONFIG_PATHS.user.settings)?,
        );

        Self::from_dto(settings_dto)
    }

    fn from_dto(settings_dto: SettingsDTO) -> Result<Settings> {
        let token = settings_dto
            .token
            .ok_or(eyre!("CLOUDFLARE_TOKEN is not set."))?;

        let zone_id = settings_dto
            .zone_id
            .ok_or(eyre!("CLOUDFLARE_ZONE_ID is not set."))?;

        Ok(Settings { token, zone_id })
    }
}

#[derive(Debug)]
struct SettingsDTO {
    token: Option<String>,
    zone_id: Option<String>,
}

impl SettingsDTO {
    fn merge(self, other: Self) -> Self {
        Self {
            token: other.token.or(self.token),
            zone_id: other.zone_id.or(self.zone_id),
        }
    }

    fn from_file_optional(path: &Path) -> Result<Self> {
        let string = read_file_optional(path);

        if let Some(body) = string {
            let dto = Self::from_string(&body)?;

            Ok(dto)
        } else {
            Ok(Self {
                token: None,
                zone_id: None,
            })
        }
    }

    fn from_string(string: &str) -> Result<Self> {
        Ok(Self::from_hashmap(Self::read_string_to_map(string)?))
    }

    fn read_string_to_map(file_body: &str) -> Result<HashMap<String, String>> {
        let mut map = HashMap::new();

        for line in file_body.lines() {
            let (k, v) = line
                .trim()
                .split_once('=')
                .ok_or(eyre!("Line '{}' is not a 'KEY=VALUE' pair.", line))?;

            if !k
                .chars()
                .all(|c| c.is_ascii_uppercase() || "-_".contains(c))
            {
                return Err(eyre!("Invalid option: '{}'\nOptions must be uppercase characters.", k));
            }

            map.insert(k.trim().to_owned(), v.trim().to_owned());
        }

        Ok(map)
    }

    fn from_hashmap(mut map: HashMap<String, String>) -> Self {
        let token = map.remove("CLOUDFLARE_TOKEN");
        let id = map.remove("CLOUDFLARE_ZONE_ID");

        Self { token, zone_id: id }
    }
}
