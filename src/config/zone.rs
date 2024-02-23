use color_eyre::eyre::eyre;
use color_eyre::Result;
use reqwest::Client;
use tracing::debug;

use crate::cloudflare::ListZonesResponse;

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

                debug!("Updating zone '{}'", name);

                Ok(zone_response.id().to_owned())
            },
        }
    }
}
