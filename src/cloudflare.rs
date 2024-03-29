use std::net::Ipv4Addr;

use color_eyre::eyre::eyre;
use color_eyre::{Report, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

pub const API_URL: &str = "https://api.cloudflare.com/client/v4";

#[derive(Deserialize, Debug)]
struct CloudFlareError {
    code: isize,
    message: String,
}

#[derive(Deserialize, Debug)]
pub struct ListZonesResponse {
    success: bool,
    errors: Vec<CloudFlareError>,
    result: Vec<ZoneResponse>,
}

impl ListZonesResponse {
    pub async fn get(client: &Client) -> Result<Self> {
        let response = client
            .get(format!("{API_URL}/zones"))
            .send()
            .await?
            .json::<Self>()
            .await?;

        if response.success {
            Ok(response)
        } else {
            Err(transform_error_responses(&response.errors))
        }
    }

    #[must_use]
    pub fn find_by_name(&self, name: &str) -> Option<&ZoneResponse> {
        self.result.iter().find(|z| z.name == name)
    }

    #[must_use]
    pub fn zones(&self) -> &[ZoneResponse] {
        &self.result
    }
}

#[derive(Deserialize, Debug)]
pub struct ZoneResponse {
    name: String,
    id: String,
}

impl ZoneResponse {
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
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
pub struct GetRecordsResponse {
    success: bool,
    errors: Vec<CloudFlareError>,
    result: Vec<DNSRecord>,
}

impl GetRecordsResponse {
    pub async fn get(client: &Client, zone_id: &str) -> Result<Self> {
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

    #[must_use]
    pub fn create_patch_record_bodies(
        &self,
        ip_address: Ipv4Addr,
    ) -> Vec<PatchRecordRequest> {
        self.result
            .iter()
            .filter_map(|record| {
                if record.record_type == "A" {
                    Some(PatchRecordRequest {
                        name: &record.name,
                        record_id: &record.id,
                        ip_address,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Serialize, Debug)]
pub struct PatchRecordRequest<'p> {
    #[serde(skip)]
    pub name: &'p str,
    #[serde(skip)]
    record_id: &'p str,
    #[serde(rename = "content")]
    ip_address: Ipv4Addr,
}

impl<'p> PatchRecordRequest<'p> {
    pub async fn patch(&self, client: &Client, zone_id: &str) -> Result<()> {
        let response = client
            .patch(format!(
                "{API_URL}/zones/{zone_id}/dns_records/{}",
                self.record_id
            ))
            .json(self)
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
}

#[derive(Deserialize, Debug)]
struct PatchRecordResponse {
    success: bool,
    errors: Vec<CloudFlareError>,
}

fn transform_error_responses(errors: &[CloudFlareError]) -> Report {
    eyre!(errors
        .iter()
        .map(|e| format!("{}: {}", e.code, e.message))
        .collect::<Vec<_>>()
        .join("\n"))
}
