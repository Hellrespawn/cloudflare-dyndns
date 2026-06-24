use color_eyre::Result;
use color_eyre::eyre::eyre;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::BUNNY_API_URL;

#[derive(Deserialize, Debug)]
pub struct BunnyRecord {
    #[serde(rename = "Id")]
    pub id: i64,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type")]
    pub record_type: u8,
    #[serde(rename = "Value")]
    pub value: String,
    #[serde(rename = "Ttl")]
    pub ttl: u32,
}

#[derive(Deserialize, Debug)]
struct GetZoneResponse {
    #[serde(rename = "Records")]
    records: Vec<BunnyRecord>,
}

pub async fn list_records(client: &Client, zone_id: i64) -> Result<Vec<BunnyRecord>> {
    let response: GetZoneResponse = client
        .get(format!("{BUNNY_API_URL}/dnszone/{zone_id}"))
        .send()
        .await?
        .error_for_status()
        .map_err(|e| eyre!("Bunny list_records failed: {e}"))?
        .json()
        .await?;

    Ok(response.records)
}

#[derive(Serialize, Debug)]
struct UpdateRecordRequest<'a> {
    #[serde(rename = "Id")]
    id: i64,
    #[serde(rename = "Type")]
    record_type: u8,
    #[serde(rename = "Value")]
    value: &'a str,
    #[serde(rename = "Name")]
    name: &'a str,
    #[serde(rename = "Ttl")]
    ttl: u32,
}

pub async fn update_record(
    client: &Client,
    zone_id: i64,
    record_id: i64,
    record_type: u8,
    name: &str,
    ttl: u32,
    new_ip: &str,
) -> Result<()> {
    client
        .post(format!("{BUNNY_API_URL}/dnszone/{zone_id}/records/{record_id}"))
        .json(&UpdateRecordRequest {
            id: record_id,
            record_type,
            value: new_ip,
            name,
            ttl,
        })
        .send()
        .await?
        .error_for_status()
        .map_err(|e| eyre!("Bunny update_record failed: {e}"))?;

    Ok(())
}
