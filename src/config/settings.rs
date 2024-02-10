use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::eyre;
use color_eyre::Result;

use super::paths::CONFIG_PATHS;
use super::{create_missing_options_error, SettingsDTO, ZoneIdentifier};
use crate::PKG_NAME;

#[derive(Debug)]
pub struct Settings {
    pub token: String,
    pub zone: ZoneIdentifier,
    pub config_path: Utf8PathBuf,
}

impl Settings {
    pub fn read() -> Result<Settings> {
        let system_settings_dto = SettingsDTO::from_file(
            &Self::get_settings_file(&CONFIG_PATHS.system),
        )?;

        let user_settings_dto = SettingsDTO::from_file(
            &Self::get_settings_file(&CONFIG_PATHS.user),
        )?;

        let dto = match (system_settings_dto, user_settings_dto) {
            (None, None) => Err(eyre!(
                "Unable to find configuration files:\n{}",
                *CONFIG_PATHS
            )),
            (None | Some(_), Some(user)) => Ok(user),
            (Some(system), None) => Ok(system),
        }?;

        Self::from_dto(dto)
    }

    pub fn get_previous_ip_file(&self) -> Utf8PathBuf {
        self.config_path.join(format!("{PKG_NAME}-previous_ip"))
    }

    fn get_settings_file(config_path: &Utf8Path) -> Utf8PathBuf {
        config_path.join(format!("{PKG_NAME}.conf"))
    }

    fn from_dto(settings_dto: SettingsDTO) -> Result<Settings> {
        let mut missing = Vec::new();

        if settings_dto.token.is_none() {
            missing.push("CLOUDFLARE_TOKEN");
        }

        if missing.is_empty() {
            Ok(Settings {
                token: settings_dto.token.as_ref().unwrap().clone(),
                config_path: settings_dto.config_path.clone(),
                zone: ZoneIdentifier::from_dto(settings_dto)?,
            })
        } else {
            Err(create_missing_options_error(&missing))
        }
    }
}
