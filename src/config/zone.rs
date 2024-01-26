use color_eyre::Result;

use super::SettingsDTO;

#[derive(Debug)]
pub enum ZoneIdentifier {
    Id(String),
    Name(String),
}

impl ZoneIdentifier {
    pub(super) fn from_dto(settings_dto: SettingsDTO) -> Result<Self> {
        match (settings_dto.zone_id, settings_dto.zone_name) {
            (None, None) => {
                Err(super::create_missing_options_error(&[
                    "CLOUDFLARE_ZONE_ID",
                    "CLOUDFLARE_NAME_ID",
                ]))
            },
            (None, Some(name)) => Ok(ZoneIdentifier::Name(name)),
            (Some(id), _) => Ok(ZoneIdentifier::Id(id)),
        }
    }
}
