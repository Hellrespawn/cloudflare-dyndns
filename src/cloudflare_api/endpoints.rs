use color_eyre::eyre::{eyre, Error};
use color_eyre::Result;
use reqwest::Client;

use super::{
    CloudFlareError, GetRecordsResponse, ListZonesResponse, PatchRecordRequest,
    PatchRecordResponse,
};

pub const API_URL: &str = "https://api.cloudflare.com/client/v4";

pub async fn list_zones(client: &Client) -> Result<ListZonesResponse> {
    let response = client
        .get(format!("{API_URL}/zones"))
        .send()
        .await?
        .json::<ListZonesResponse>()
        .await?;

    if response.success {
        Ok(response)
    } else {
        Err(transform_error_responses(&response.errors))
    }
}

pub async fn get_records(
    client: &Client,
    zone_id: &str,
) -> Result<GetRecordsResponse> {
    let response = client
        .get(format!("{API_URL}/zones/{zone_id}/dns_records"))
        .send()
        .await?
        .json::<GetRecordsResponse>()
        .await?;

    if response.success {
        Ok(response)
    } else {
        Err(transform_error_responses(&response.errors))
    }
}
pub async fn patch_records(
    client: &Client,
    zone_id: &str,
    record_id: &str,
    request: PatchRecordRequest,
) -> Result<()> {
    let response = client
        .patch(format!("{API_URL}/zones/{zone_id}/dns_records/{record_id}",))
        .json(&request)
        .send()
        .await?
        .json::<PatchRecordResponse>()
        .await?;

    if response.success {
        Ok(())
    } else {
        Err(transform_error_responses(&response.errors))
    }
}

fn transform_error_responses(errors: &[CloudFlareError]) -> Error {
    eyre!(errors
        .iter()
        .map(|e| format!("{}: {}", e.code, e.message))
        .collect::<Vec<_>>()
        .join("\n"))
}
