use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ConfigPaths {
    pub system: ConfigPath,
    pub user: ConfigPath,
}

impl std::fmt::Display for ConfigPaths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "  {}\n  {}",
            CONFIG_PATHS.system.settings, CONFIG_PATHS.user.settings
        )
    }
}

#[derive(Debug)]
pub struct ConfigPath {
    pub settings: Utf8PathBuf,
    pub previous_ip: Utf8PathBuf,
}

pub static CONFIG_PATHS: Lazy<ConfigPaths> = Lazy::new(|| {
    get_config_paths().expect("Unable to init configuration paths.")
});

pub fn get_config_paths() -> Result<ConfigPaths> {
    let pkg_name: &str = env!("CARGO_PKG_NAME");

    let settings_file_name = format!("{pkg_name}.conf");
    let previous_ip_file_name = format!("{pkg_name}-previous_ip");

    let system_config_directory = Utf8PathBuf::from(format!("/etc/{pkg_name}"));

    let user_config_directory: Utf8PathBuf = dirs::config_dir()
        .ok_or(eyre!("Unable to read user config directory."))?
        .join(pkg_name)
        .try_into()?;

    Ok(ConfigPaths {
        system: ConfigPath {
            settings: system_config_directory.join(&settings_file_name),
            previous_ip: system_config_directory.join(&previous_ip_file_name),
        },
        user: ConfigPath {
            settings: user_config_directory.join(settings_file_name),
            previous_ip: user_config_directory.join(previous_ip_file_name),
        },
    })
}

#[derive(Debug)]
pub struct Settings {
    pub token: String,
    pub zone_id: String,
}

impl Settings {
    pub fn read() -> Result<Settings> {
        let system_settings_dto =
            SettingsDTO::from_file(&CONFIG_PATHS.system.settings)?;

        let user_settings_dto =
            SettingsDTO::from_file(&CONFIG_PATHS.user.settings)?;

        if system_settings_dto.is_none() && user_settings_dto.is_none() {
            Err(eyre!(
                "Unable to find configuration files:\n{}",
                *CONFIG_PATHS
            ))
        } else {
            Self::from_dto(
                system_settings_dto
                    .unwrap_or_default()
                    .merge(user_settings_dto.unwrap_or_default()),
            )
        }
    }

    fn from_dto(settings_dto: SettingsDTO) -> Result<Settings> {
        let mut missing = Vec::new();

        if settings_dto.token.is_none() {
            missing.push("CLOUDFLARE_TOKEN");
        }

        if settings_dto.zone_id.is_none() {
            missing.push("CLOUDFLARE_ZONE_ID");
        }

        if missing.is_empty() {
            Ok(Settings {
                token: settings_dto.token.unwrap(),
                zone_id: settings_dto.zone_id.unwrap(),
            })
        } else {
            Err(eyre!(
                "Unable to read variables: {}\nCheck your confguration files:\n{}",
                missing.join(", "),
                *CONFIG_PATHS
            ))
        }
    }
}

#[derive(Debug, Default)]
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

    fn from_file(path: &Utf8Path) -> Result<Option<Self>> {
        if path.is_file() {
            let map = dotenvy::from_path_iter(path)?
                .map(|pair| Ok(pair?))
                .collect::<Result<HashMap<String, String>>>()?;

            Ok(Some(Self::from_hashmap(map)))
        } else if path.exists() {
            Err(eyre!("Path exists but is not a file: {}", path))
        } else {
            Ok(None)
        }
    }

    fn from_hashmap(mut map: HashMap<String, String>) -> Self {
        let token = map.remove("CLOUDFLARE_TOKEN");
        let id = map.remove("CLOUDFLARE_ZONE_ID");

        Self { token, zone_id: id }
    }
}
