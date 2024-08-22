use serde::{Deserialize, Serialize};
use tabled::Tabled;

pub mod endpoints;

#[derive(Deserialize, Debug)]
pub struct CloudFlareError {
    pub code: isize,
    pub message: String,
}

#[derive(Deserialize, Debug)]
pub struct ListZonesResponse {
    pub success: bool,
    pub errors: Vec<CloudFlareError>,
    pub result: Vec<ZoneResponse>,
}

#[derive(Deserialize, Debug)]
pub struct ZoneResponse {
    pub name: String,
    pub id: String,
}
#[derive(Deserialize, Debug)]
pub struct GetRecordsResponse {
    pub success: bool,
    pub errors: Vec<CloudFlareError>,
    pub result: Vec<DNSRecordResponse>,
}

#[derive(Deserialize, Serialize, Debug, Tabled)]
pub struct DNSRecordResponse {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub content: String,
}

#[derive(Serialize, Debug)]
pub struct PatchRecordRequest {
    pub content: String,
}

#[derive(Deserialize, Debug)]
struct PatchRecordResponse {
    pub success: bool,
    pub errors: Vec<CloudFlareError>,
}
