use color_eyre::Result;
use reqwest::Client;
use serde::Deserialize;

use super::{transform_error_responses, CloudFlareError, API_URL};

#[derive(Deserialize, Debug)]
struct ListZonesResponse {
    success: bool,
    errors: Vec<CloudFlareError>,
    result: Vec<ZoneResponse>,
}

#[derive(Deserialize, Debug)]
pub struct ZoneResponse {
    pub name: String,
    pub id: String,
}

pub async fn list_zones(client: &Client) -> Result<Vec<ZoneResponse>> {
    let response = client
        .get(format!("{API_URL}/zones"))
        .send()
        .await?
        .json::<ListZonesResponse>()
        .await?;

    if response.success {
        Ok(response.result)
    } else {
        Err(transform_error_responses(&response.errors))
    }
}
