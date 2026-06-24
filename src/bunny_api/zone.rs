use color_eyre::Result;
use color_eyre::eyre::eyre;
use reqwest::Client;
use serde::Deserialize;

use super::BUNNY_API_URL;

#[derive(Deserialize, Debug)]
pub struct BunnyZone {
    #[serde(rename = "Id")]
    pub id: i64,
    #[serde(rename = "Domain")]
    pub domain: String,
}

#[derive(Deserialize, Debug)]
struct ListZonesResponse {
    #[serde(rename = "Items")]
    items: Vec<BunnyZone>,
    #[serde(rename = "HasMoreItems")]
    has_more_items: bool,
}

pub async fn list_zones(client: &Client) -> Result<Vec<BunnyZone>> {
    let mut all_zones = Vec::new();
    let mut page = 1u32;

    loop {
        let response: ListZonesResponse = client
            .get(format!("{BUNNY_API_URL}/dnszone?page={page}&perPage=1000"))
            .send()
            .await?
            .error_for_status()
            .map_err(|e| eyre!("Bunny list_zones failed: {e}"))?
            .json()
            .await?;

        let has_more = response.has_more_items;
        all_zones.extend(response.items);

        if !has_more {
            break;
        }
        page += 1;
    }

    Ok(all_zones)
}
