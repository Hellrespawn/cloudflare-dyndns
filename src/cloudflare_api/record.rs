use color_eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

use super::{API_URL, CloudFlareError, transform_error_responses};

#[derive(Deserialize, Debug)]
struct GetRecordsResponse {
    success: bool,
    errors: Vec<CloudFlareError>,
    result: Vec<DNSRecord>,
}

#[derive(Deserialize, Debug)]
pub struct DNSRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: DNSRecordType,
    pub content: String,
}

#[derive(
    Deserialize, Debug, PartialEq, Eq, Default, Copy, Clone, EnumString, Display,
)]
pub enum DNSRecordType {
    #[default]
    A,
    AAAA,
    MX,
    TXT,
    SRV,
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

    if response.success {
        Ok(response.result)
    } else {
        Err(transform_error_responses(&response.errors))
    }
}

#[derive(Serialize, Debug)]
pub struct PatchRecordRequest<'c> {
    content: &'c str,
}

impl<'c> PatchRecordRequest<'c> {
    #[must_use]
    fn new(content: &'c str) -> Self {
        Self { content }
    }
}

#[derive(Deserialize, Debug)]
struct PatchRecordResponse {
    success: bool,
    errors: Vec<CloudFlareError>,
}

pub async fn patch_record(
    client: &Client,
    zone_id: &str,
    record_id: &str,
    content: &str,
) -> Result<()> {
    let response = client
        .patch(format!("{API_URL}/zones/{zone_id}/dns_records/{record_id}",))
        .json(&PatchRecordRequest::new(content))
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
