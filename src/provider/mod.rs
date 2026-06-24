//! Provider abstraction over Cloudflare and bunny.net DNS APIs.
//!
//! Both providers are wrapped behind [`DnsProvider`] and use the same [`DnsRecord`] and [`Zone`]
//! types. The differences between the two APIs are handled in each provider module:
//!
//! | Aspect          | Cloudflare                            | bunny.net                                 |
//! |-----------------|---------------------------------------|-------------------------------------------|
//! | Auth header     | `Authorization: Bearer {token}`       | `AccessKey: {token}`                      |
//! | Record name     | FQDN (`www.example.com`)              | Bare subdomain (`www`); apex as `""` or `"@"` |
//! | Record type     | String (`"A"`, `"AAAA"`, …)           | Integer (`0`=A, `1`=AAAA, `2`=CNAME, `3`=TXT, `4`=MX, `8`=SRV) |
//! | Update method   | `PATCH` with changed fields only      | `POST` with full record body incl. TTL    |
//! | Zone listing    | Cursor-based pagination               | Page/perPage with `HasMoreItems` flag     |
//!
//! **Name normalisation**: to give callers a consistent view, `BunnyProvider::list_records`
//! converts bare subdomain names to FQDNs (`www` → `www.example.com`, `""` → `example.com`).
//! `BunnyProvider::update_record` reverses this before calling the API.

pub mod cloudflare;
pub mod bunny;

use color_eyre::Result;
use serde::Deserialize;
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone)]
pub struct Zone {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct DnsRecord {
    pub id: String,
    pub name: String,
    pub record_type: DnsRecordType,
    pub content: String,
    pub ttl: Option<u32>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Default, Copy, Clone, EnumString, Display)]
pub enum DnsRecordType {
    #[default]
    A,
    AAAA,
    MX,
    TXT,
    SRV,
    CNAME,
    #[serde(other)]
    MISC,
}

impl From<u8> for DnsRecordType {
    fn from(t: u8) -> Self {
        match t {
            0 => Self::A,
            1 => Self::AAAA,
            2 => Self::CNAME,
            3 => Self::TXT,
            4 => Self::MX,
            8 => Self::SRV,
            _ => Self::MISC,
        }
    }
}

pub trait DnsProvider {
    async fn list_zones(&self) -> Result<Vec<Zone>>;
    async fn list_records(&self, zone: &Zone) -> Result<Vec<DnsRecord>>;
    async fn update_record(
        &self,
        zone: &Zone,
        record: &DnsRecord,
        new_ip: &str,
    ) -> Result<()>;
}
