use anyhow::{bail, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

const API_URL: &str = "https://api.cloudflare.com/client/v4";

#[derive(Deserialize, Debug)]
struct ErrorResponse {
    code: isize,
    message: String,
}

#[derive(Deserialize, Debug)]
pub struct DNSRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub content: String,
}

#[derive(Deserialize, Debug)]
struct GetRecordsResponse {
    success: bool,
    errors: Vec<ErrorResponse>,
    result: Vec<DNSRecord>,
}

#[derive(Serialize, Debug)]
pub struct PatchRecordBody<'p> {
    content: &'p str,
}

#[derive(Deserialize, Debug)]
struct PatchRecordResponse {
    success: bool,
    errors: Vec<ErrorResponse>,
}

fn transform_error_responses(errors: &[ErrorResponse]) -> String {
    errors
        .iter()
        .map(|e| format!("{}: {}", e.code, e.message))
        .collect::<Vec<_>>()
        .join("\n")
}

pub async fn get_records(
    client: &Client,
    zone_id: &str,
) -> Result<Vec<DNSRecord>> {
    let response = client
        .get(format!("{API_URL}/zones/{zone_id}/dns_records"))
        .send()
        .await?
        .json::<GetRecordsResponse>()
        .await?;

    if !response.success {
        bail!(transform_error_responses(&response.errors))
    }

    Ok(response.result)
}

pub async fn patch_record(
    client: &Client,
    zone_id: &str,
    record_id: &str,
    ip_address: &str,
) -> Result<()> {
    let response = client
        .patch(format!("{API_URL}/zones/{zone_id}/dns_records/{record_id}"))
        .json(&PatchRecordBody {
            content: ip_address,
        })
        .send()
        .await?
        .json::<PatchRecordResponse>()
        .await?;

    if !response.success {
        bail!(transform_error_responses(&response.errors))
    }

    Ok(())
}
