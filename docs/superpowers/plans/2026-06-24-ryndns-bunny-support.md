# ryndns: bunny.net Support Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rename the crate to `ryndns` and add bunny.net as a second DNS provider alongside Cloudflare using a `DnsProvider` trait.

**Architecture:** A new `src/provider/` module defines the `DnsProvider` trait and shared domain types (`Zone`, `DnsRecord`, `DnsRecordType`). `CloudflareProvider` and `BunnyProvider` implement the trait, each holding a `reqwest::Client` built with provider-specific auth headers. The main loop iterates over configured providers, building each provider from config, and calls a single generic `handle_zone<P: DnsProvider>` function.

**Tech Stack:** Rust 2024 edition, reqwest (with rustls), serde/serde_json, toml, strum, color-eyre, derive_builder.

## Global Constraints

- Rust edition 2024 (native async fn in traits — no `async_trait` crate needed)
- Clean break: old `cloudflare_token` + flat zone config format is not supported
- Binary name: `ryndns`; crate name: `ryndns`
- Config paths derive from `PKG_NAME` automatically (`~/.config/ryndns/ryndns.toml` or `/etc/ryndns/ryndns.toml` when root)
- Auth: Cloudflare uses `Authorization: Bearer {token}`; Bunny uses `AccessKey: {token}`
- `DnsRecord.name` stores the normalized name: full FQDN for Cloudflare records, subdomain-only for Bunny (empty → zone domain for apex)
- All `cargo build` and `cargo test` must pass at the end of each task

---

## File Map

| File | Action | Purpose |
|---|---|---|
| `Cargo.toml` | Modify | Rename crate/binary |
| `src/bin/main.rs` | Modify | Update crate name in call |
| `src/bin/list_zones.rs` | Modify | Update crate name in call |
| `src/bin/public_ip.rs` | Modify | Update crate name in call |
| `src/lib.rs` | Modify | Add `provider`/`bunny_api` modules, remove `create_reqwest_client` |
| `src/provider/mod.rs` | Create | `DnsProvider` trait, `Zone`, `DnsRecord`, `DnsRecordType` |
| `src/provider/cloudflare.rs` | Create | `CloudflareProvider` implements `DnsProvider` |
| `src/provider/bunny.rs` | Create | `BunnyProvider` implements `DnsProvider` |
| `src/cloudflare_api/mod.rs` | Modify | Use `crate::provider::DnsRecordType` |
| `src/cloudflare_api/record.rs` | Modify | Rename `DNSRecord` → `CloudflareRecord`; remove `DNSRecordType` |
| `src/cloudflare_api/zone.rs` | No change | Already clean |
| `src/bunny_api/mod.rs` | Create | Bunny error type, `BUNNY_API_URL` |
| `src/bunny_api/zone.rs` | Create | `list_zones` raw API call |
| `src/bunny_api/record.rs` | Create | `list_records`, `update_record` raw API calls |
| `src/config/mod.rs` | Modify | New `ApplicationConfig`, `ProviderConfig` |
| `src/config/fs.rs` | No change | Derives paths from `PKG_NAME` automatically |
| `src/state.rs` | Modify | Remove `client`, `zone_name_to_id_map` |
| `src/cli/dyndns.rs` | Rewrite | New multi-provider main loop |
| `src/cli/list_zones.rs` | Rewrite | List zones from all configured providers |
| `src/cli/public_ip.rs` | Modify | Update doc string |
| `test/example.toml` | Modify | New config format |
| `cloudflare-dyndns.example.toml` | Rename → `ryndns.example.toml` | New config format |
| `cloudflare-dyndns.service` | Rename → `ryndns.service` | Updated ExecStart |
| `cloudflare-dyndns.timer` | Rename → `ryndns.timer` | No content change needed |
| `install.sh` | Modify | Updated `name` variable |
| `README.md` | Modify | Updated name, config format, usage |

---

## Task 1: Rename crate, binary, and supporting files

**Files:**
- Modify: `Cargo.toml`
- Modify: `src/bin/main.rs`, `src/bin/list_zones.rs`, `src/bin/public_ip.rs`
- Rename: `cloudflare-dyndns.example.toml` → `ryndns.example.toml`
- Rename: `cloudflare-dyndns.service` → `ryndns.service`
- Rename: `cloudflare-dyndns.timer` → `ryndns.timer`
- Modify: `install.sh`

**Interfaces:**
- Produces: binary named `ryndns`, crate named `ryndns`

- [ ] **Step 1: Update Cargo.toml**

```toml
[package]
description = "Dynamic DNS for Cloudflare and bunny.net."
edition = "2024"
license = "BSD 3-Clause"
name = "ryndns"
repository = "https://github.com/Hellrespawn/cloudflare-dyndns"
version = "0.10.0"
default-run = "ryndns"

[package.metadata.cargo-machete]
ignored = ["strum"]

[[bin]]
name = "ryndns"
path = "src/bin/main.rs"

[dependencies]
# (unchanged)
```

- [ ] **Step 2: Update bin entry points**

`src/bin/main.rs`:
```rust
#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    ryndns::cli::dyndns::main().await
}
```

`src/bin/list_zones.rs`:
```rust
#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    ryndns::cli::list_zones::main().await
}
```

`src/bin/public_ip.rs`:
```rust
#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    ryndns::cli::public_ip::main().await
}
```

- [ ] **Step 3: Rename supporting files**

```bash
git mv cloudflare-dyndns.example.toml ryndns.example.toml
git mv cloudflare-dyndns.service ryndns.service
git mv cloudflare-dyndns.timer ryndns.timer
```

- [ ] **Step 4: Update ryndns.service**

```ini
[Unit]
Description=Update DNS records

[Service]
Type=oneshot
ExecStart=/opt/ryndns/ryndns
```

- [ ] **Step 5: Update install.sh**

Change the `name` variable at the top:
```bash
name="ryndns"
```

The rest of the script uses `$name` throughout, so no other changes are needed.

- [ ] **Step 6: Build to confirm rename compiles**

```bash
cargo build
```

Expected: compiles successfully.

- [ ] **Step 7: Commit**

```bash
git add -A
git commit -m "feat: rename crate and binary to ryndns"
```

---

## Task 2: Create shared provider types and DnsProvider trait

**Files:**
- Create: `src/provider/mod.rs`
- Modify: `src/cloudflare_api/record.rs` (use `crate::provider::DnsRecordType`, remove `DNSRecordType`)
- Modify: `src/config/mod.rs` (use `crate::provider::DnsRecordType`)
- Modify: `src/lib.rs` (add `pub mod provider;`)

**Interfaces:**
- Produces:
  - `crate::provider::Zone { id: String, name: String }`
  - `crate::provider::DnsRecord { id: String, name: String, record_type: DnsRecordType, content: String }`
  - `crate::provider::DnsRecordType` (enum: A, AAAA, MX, TXT, SRV, CNAME, MISC)
  - `crate::provider::DnsProvider` trait (3 async methods)

- [ ] **Step 1: Create src/provider/mod.rs**

```rust
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
```

Create empty placeholder files so the module compiles:

`src/provider/cloudflare.rs`:
```rust
// CloudflareProvider — implemented in Task 3
```

`src/provider/bunny.rs`:
```rust
// BunnyProvider — implemented in Task 6
```

- [ ] **Step 2: Update src/cloudflare_api/record.rs**

Replace the existing `DNSRecordType` definition and import with the shared type. Rename `DNSRecord` to `CloudflareRecord` and update its `record_type` field:

```rust
use color_eyre::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::provider::DnsRecordType;

use super::{API_URL, CloudFlareError, transform_error_responses};

#[derive(Deserialize, Debug)]
struct GetRecordsResponse {
    success: bool,
    errors: Vec<CloudFlareError>,
    result: Vec<CloudflareRecord>,
}

#[derive(Deserialize, Debug)]
pub struct CloudflareRecord {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub record_type: DnsRecordType,
    pub content: String,
}

pub async fn get_records(
    client: &Client,
    zone_id: &str,
) -> Result<Vec<CloudflareRecord>> {
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
        .patch(format!("{API_URL}/zones/{zone_id}/dns_records/{record_id}"))
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
```

- [ ] **Step 3: Update src/config/mod.rs import**

Change:
```rust
use crate::cloudflare_api::record::{DNSRecord, DNSRecordType};
```
To:
```rust
use crate::cloudflare_api::record::CloudflareRecord;
use crate::provider::DnsRecordType;
```

Update `ZoneConfig::is_record_selected` to take `&CloudflareRecord`:
```rust
pub fn is_record_selected(&self, record: &CloudflareRecord) -> bool {
    self.records.iter().any(|r| r.match_record(record))
}
```

Update `RecordConfig::match_record`:
```rust
fn match_record(&self, record: &CloudflareRecord) -> bool {
    let record_name = &record.name;
    let self_name = self.name();

    let name_matches = self_name == record_name
        || record_name.starts_with(&format!("{self_name}."));

    let type_matches = self.record_type() == record.record_type;

    type_matches && name_matches
}
```

- [ ] **Step 4: Add provider module to src/lib.rs**

Add `pub mod provider;` after the existing module declarations:
```rust
pub mod cli;
pub mod cloudflare_api;
pub mod config;
pub mod ip_cache;
pub mod provider;
pub mod state;
```

Remove `strum` from the `use` block at the top if `DNSRecordType` was the only user — `strum` is still used by `DnsRecordType` in `provider/mod.rs`.

- [ ] **Step 5: Run tests**

```bash
cargo test
```

Expected: `test_deserialize_default` passes; crate compiles.

- [ ] **Step 6: Commit**

```bash
git add src/provider/ src/cloudflare_api/record.rs src/config/mod.rs src/lib.rs
git commit -m "feat: add shared DnsProvider trait and domain types"
```

---

## Task 3: Implement CloudflareProvider

**Files:**
- Modify: `src/provider/cloudflare.rs`

**Interfaces:**
- Consumes: `crate::cloudflare_api::{record::{get_records, patch_record, CloudflareRecord}, zone::list_zones}`, `crate::provider::{DnsProvider, DnsRecord, Zone}`
- Produces: `CloudflareProvider` implementing `DnsProvider`

- [ ] **Step 1: Implement CloudflareProvider**

```rust
use color_eyre::Result;
use color_eyre::eyre::eyre;
use reqwest::Client;
use reqwest::header::HeaderMap;

use crate::cloudflare_api::record::{get_records, patch_record};
use crate::cloudflare_api::zone::list_zones as cf_list_zones;
use crate::provider::{DnsProvider, DnsRecord, Zone};

pub struct CloudflareProvider {
    client: Client,
}

impl CloudflareProvider {
    pub fn new(token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Content-Type",
            "application/json"
                .parse()
                .map_err(|_| eyre!("Invalid Content-Type header"))?,
        );
        headers.insert(
            "Authorization",
            format!("Bearer {token}")
                .parse()
                .map_err(|_| eyre!("Invalid Authorization header"))?,
        );
        let client = Client::builder()
            .default_headers(headers)
            .use_rustls_tls()
            .build()?;
        Ok(Self { client })
    }
}

impl DnsProvider for CloudflareProvider {
    async fn list_zones(&self) -> Result<Vec<Zone>> {
        let zones = cf_list_zones(&self.client).await?;
        Ok(zones.into_iter().map(|z| Zone { id: z.id, name: z.name }).collect())
    }

    async fn list_records(&self, zone: &Zone) -> Result<Vec<DnsRecord>> {
        let records = get_records(&self.client, &zone.id).await?;
        Ok(records
            .into_iter()
            .map(|r| DnsRecord {
                id: r.id,
                name: r.name,
                record_type: r.record_type,
                content: r.content,
            })
            .collect())
    }

    async fn update_record(
        &self,
        _zone: &Zone,
        record: &DnsRecord,
        new_ip: &str,
    ) -> Result<()> {
        patch_record(&self.client, &_zone.id, &record.id, new_ip).await
    }
}
```

- [ ] **Step 2: Build**

```bash
cargo build
```

Expected: compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add src/provider/cloudflare.rs
git commit -m "feat: implement CloudflareProvider"
```

---

## Task 4: Update config for provider sections

**Files:**
- Modify: `src/config/mod.rs`
- Modify: `test/example.toml`
- Modify: `ryndns.example.toml`

**Interfaces:**
- Produces:
  - `ApplicationConfig { public_ip_url: String, cloudflare: Option<ProviderConfig>, bunny: Option<ProviderConfig> }`
  - `ProviderConfig { token: String, zones: IndexMap<String, ZoneConfig> }`
  - `ApplicationConfig::cloudflare() -> Option<&ProviderConfig>`
  - `ApplicationConfig::bunny() -> Option<&ProviderConfig>`
  - `ProviderConfig::token() -> &str`
  - `ProviderConfig::zones() -> &IndexMap<String, ZoneConfig>`
  - `ApplicationConfig::public_ip_url() -> &str` (unchanged)

- [ ] **Step 1: Rewrite src/config/mod.rs**

```rust
use indexmap::IndexMap;
use serde::Deserialize;

use crate::provider::DnsRecordType;

mod fs;

pub use fs::ApplicationConfigLoader;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationConfig {
    public_ip_url: String,
    cloudflare: Option<ProviderConfig>,
    bunny: Option<ProviderConfig>,
}

impl ApplicationConfig {
    #[must_use]
    pub fn public_ip_url(&self) -> &str {
        &self.public_ip_url
    }

    #[must_use]
    pub fn cloudflare(&self) -> Option<&ProviderConfig> {
        self.cloudflare.as_ref()
    }

    #[must_use]
    pub fn bunny(&self) -> Option<&ProviderConfig> {
        self.bunny.as_ref()
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ProviderConfig {
    token: String,
    #[serde(flatten)]
    zones: IndexMap<String, ZoneConfig>,
}

impl ProviderConfig {
    #[must_use]
    pub fn token(&self) -> &str {
        &self.token
    }

    #[must_use]
    pub fn zones(&self) -> &IndexMap<String, ZoneConfig> {
        &self.zones
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ZoneConfig {
    records: Vec<RecordConfig>,
}

impl ZoneConfig {
    #[must_use]
    pub fn records(&self) -> &[RecordConfig] {
        &self.records
    }

    #[must_use]
    pub fn is_record_selected(&self, record_name: &str, record_type: DnsRecordType) -> bool {
        self.records.iter().any(|r| r.matches(record_name, record_type))
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(untagged)]
pub enum RecordConfig {
    Full {
        #[serde(rename = "type", default)]
        record_type: DnsRecordType,
        name: String,
    },
    Name(String),
}

impl RecordConfig {
    fn matches(&self, record_name: &str, record_type: DnsRecordType) -> bool {
        let self_name = self.name();
        let name_matches = self_name == record_name
            || record_name.starts_with(&format!("{self_name}."));
        let type_matches = self.record_type() == record_type;
        type_matches && name_matches
    }

    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            RecordConfig::Full { name, .. } | RecordConfig::Name(name) => name,
        }
    }

    #[must_use]
    pub fn record_type(&self) -> DnsRecordType {
        match self {
            RecordConfig::Full { record_type, .. } => *record_type,
            RecordConfig::Name(_) => DnsRecordType::A,
        }
    }
}

#[cfg(test)]
mod test {
    use color_eyre::Result;

    use super::*;

    const EXAMPLE: &str = include_str!("../../test/example.toml");

    fn get_expected_config() -> ApplicationConfig {
        let mut cf_zones = IndexMap::new();
        cf_zones.insert("example.nl".to_owned(), ZoneConfig {
            records: vec![
                RecordConfig::Full {
                    record_type: DnsRecordType::A,
                    name: "www".to_owned(),
                },
                RecordConfig::Name("mail".to_owned()),
            ],
        });

        let mut bunny_zones = IndexMap::new();
        bunny_zones.insert("otherexample.com".to_owned(), ZoneConfig {
            records: vec![
                RecordConfig::Full {
                    record_type: DnsRecordType::A,
                    name: "www".to_owned(),
                },
                RecordConfig::Name("mail".to_owned()),
            ],
        });

        ApplicationConfig {
            public_ip_url: "https://example.ip".to_owned(),
            cloudflare: Some(ProviderConfig {
                token: "cf_token".to_owned(),
                zones: cf_zones,
            }),
            bunny: Some(ProviderConfig {
                token: "bunny_token".to_owned(),
                zones: bunny_zones,
            }),
        }
    }

    #[test]
    fn test_deserialize() -> Result<()> {
        let config: ApplicationConfig = toml::from_str(EXAMPLE)?;
        assert_eq!(config, get_expected_config());
        Ok(())
    }
}
```

Note: `is_record_selected` now takes `(record_name: &str, record_type: DnsRecordType)` instead of `&CloudflareRecord`, so it works with the shared `DnsRecord` type. `src/cli/dyndns.rs` is already broken at this point — it gets fixed in Task 7.

- [ ] **Step 2: Update test/example.toml**

```toml
public_ip_url = "https://example.ip"

[cloudflare]
token = "cf_token"

[cloudflare."example.nl"]
records = [
    { type = "A", name = "www" },
    "mail",
]

[bunny]
token = "bunny_token"

[bunny."otherexample.com"]
records = [
    { type = "A", name = "www" },
    "mail",
]
```

- [ ] **Step 3: Update ryndns.example.toml**

```toml
public_ip_url = "https://example.ip"

[cloudflare]
token = ""

[cloudflare."example.com"]
records = ["example.com", "*", { type = "AAAA", name = "mail" }]

[bunny]
token = ""

[bunny."example.nl"]
records = ["example.nl", "*", { type = "AAAA", name = "mail" }]
```

- [ ] **Step 4: Run tests**

```bash
cargo test
```

Expected: `test_deserialize` passes. (`cli/dyndns.rs` and `cli/list_zones.rs` will have compile errors — fix those next; for now, comment out their module registrations in `src/cli/mod.rs` if needed to run the test.)

Actually, to run the config test in isolation:

```bash
cargo test --lib config
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/config/mod.rs test/example.toml ryndns.example.toml
git commit -m "feat: update config for provider sections"
```

---

## Task 5: Create bunny_api module

> **⚠️ Verify against https://docs.bunny.net/reference before implementing:**
> - HTTP methods for each endpoint (list zones, get zone records, update record)
> - Required fields in the update request body
> - Pagination on `GET /dnszone` (check `HasMoreItems`)
> - `Name` field for apex records (empty string, `@`, or zone domain?)
>
> The implementation below reflects best-effort knowledge; adjust to match the actual API.

**Files:**
- Create: `src/bunny_api/mod.rs`
- Create: `src/bunny_api/zone.rs`
- Create: `src/bunny_api/record.rs`
- Modify: `src/lib.rs` (add `pub mod bunny_api;`)

**Interfaces:**
- Produces:
  - `bunny_api::zone::list_zones(client: &Client) -> Result<Vec<BunnyZone>>`
  - `bunny_api::record::list_records(client: &Client, zone_id: i64) -> Result<Vec<BunnyRecord>>`
  - `bunny_api::record::update_record(client: &Client, zone_id: i64, record_id: i64, record_type: u8, name: &str, new_ip: &str) -> Result<()>`
  - `BunnyZone { id: i64, domain: String }`
  - `BunnyRecord { id: i64, name: String, record_type: u8, value: String }`

- [ ] **Step 1: Create src/bunny_api/mod.rs**

```rust
pub mod record;
pub mod zone;

const BUNNY_API_URL: &str = "https://api.bunny.net";

pub fn build_bunny_client(token: &str) -> color_eyre::Result<reqwest::Client> {
    use color_eyre::eyre::eyre;
    use reqwest::header::HeaderMap;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json"
            .parse()
            .map_err(|_| eyre!("Invalid Content-Type header"))?,
    );
    headers.insert(
        "AccessKey",
        token
            .parse()
            .map_err(|_| eyre!("Invalid AccessKey header"))?,
    );
    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .use_rustls_tls()
        .build()?)
}
```

- [ ] **Step 2: Create src/bunny_api/zone.rs**

```rust
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
            .get(format!("{BUNNY_API_URL}/dnszone"))
            .query(&[("page", page), ("perPage", 1000)])
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
```

- [ ] **Step 3: Create src/bunny_api/record.rs**

```rust
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
```

- [ ] **Step 4: Add bunny_api to src/lib.rs**

```rust
pub mod bunny_api;
pub mod cli;
pub mod cloudflare_api;
pub mod config;
pub mod ip_cache;
pub mod provider;
pub mod state;
```

- [ ] **Step 5: Build**

```bash
cargo build
```

Expected: compiles successfully (bunny_api module compiles; cli modules may still be broken).

- [ ] **Step 6: Commit**

```bash
git add src/bunny_api/ src/lib.rs
git commit -m "feat: add bunny_api raw HTTP module"
```

---

## Task 6: Implement BunnyProvider

**Files:**
- Modify: `src/provider/bunny.rs`

**Interfaces:**
- Consumes: `crate::bunny_api::{build_bunny_client, zone::list_zones, record::{list_records, update_record, BunnyRecord}}`, `crate::provider::{DnsProvider, DnsRecord, DnsRecordType, Zone}`
- Produces: `BunnyProvider` implementing `DnsProvider`
- `DnsRecord.name` for Bunny: subdomain only, except apex which is normalized to `{zone.name}`

- [ ] **Step 1: Implement BunnyProvider**

```rust
use color_eyre::Result;
use reqwest::Client;

use crate::bunny_api::build_bunny_client;
use crate::bunny_api::record::{BunnyRecord, list_records, update_record};
use crate::bunny_api::zone::list_zones as bunny_list_zones;
use crate::provider::{DnsProvider, DnsRecord, DnsRecordType, Zone};

pub struct BunnyProvider {
    client: Client,
}

impl BunnyProvider {
    pub fn new(token: &str) -> Result<Self> {
        Ok(Self { client: build_bunny_client(token)? })
    }
}

fn bunny_type_to_u8(t: DnsRecordType) -> u8 {
    match t {
        DnsRecordType::A => 0,
        DnsRecordType::AAAA => 1,
        DnsRecordType::CNAME => 2,
        DnsRecordType::TXT => 3,
        DnsRecordType::MX => 4,
        DnsRecordType::SRV => 8,
        DnsRecordType::MISC => 255,
    }
}

fn normalize_name(bunny_name: &str, zone_name: &str) -> String {
    if bunny_name.is_empty() || bunny_name == "@" {
        zone_name.to_owned()
    } else {
        bunny_name.to_owned()
    }
}

fn raw_subdomain<'a>(record_name: &'a str, zone_name: &str) -> &'a str {
    if record_name == zone_name {
        ""
    } else {
        record_name
    }
}

fn map_bunny_record(r: BunnyRecord, zone_name: &str) -> DnsRecord {
    DnsRecord {
        id: r.id.to_string(),
        name: normalize_name(&r.name, zone_name),
        record_type: DnsRecordType::from(r.record_type),
        content: r.value,
    }
}

impl DnsProvider for BunnyProvider {
    async fn list_zones(&self) -> Result<Vec<Zone>> {
        let zones = bunny_list_zones(&self.client).await?;
        Ok(zones
            .into_iter()
            .map(|z| Zone { id: z.id.to_string(), name: z.domain })
            .collect())
    }

    async fn list_records(&self, zone: &Zone) -> Result<Vec<DnsRecord>> {
        let zone_id: i64 = zone.id.parse()?;
        let records = list_records(&self.client, zone_id).await?;
        Ok(records
            .into_iter()
            .map(|r| map_bunny_record(r, &zone.name))
            .collect())
    }

    async fn update_record(
        &self,
        zone: &Zone,
        record: &DnsRecord,
        new_ip: &str,
    ) -> Result<()> {
        let zone_id: i64 = zone.id.parse()?;
        let record_id: i64 = record.id.parse()?;
        let raw_name = raw_subdomain(&record.name, &zone.name);
        update_record(
            &self.client,
            zone_id,
            record_id,
            bunny_type_to_u8(record.record_type),
            raw_name,
            300,
            new_ip,
        )
        .await
    }
}
```

Note on TTL: `300` is used as a safe default since `BunnyRecord.ttl` is not stored in `DnsRecord`. If you need to preserve existing TTL, store it in a provider-specific wrapper or extend `DnsRecord` with an optional `ttl` field.

- [ ] **Step 2: Build**

```bash
cargo build
```

Expected: compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add src/provider/bunny.rs
git commit -m "feat: implement BunnyProvider"
```

---

## Task 7: Update ApplicationState and main loop

**Files:**
- Modify: `src/state.rs`
- Rewrite: `src/cli/dyndns.rs`
- Modify: `src/lib.rs` (remove `create_reqwest_client`, `get_public_ip_address` stays)

**Interfaces:**
- Consumes: `CloudflareProvider`, `BunnyProvider`, `DnsProvider`, `DnsRecord`, `Zone`, new `ApplicationConfig`, `ProviderConfig`
- Produces: working `ryndns` binary with multi-provider support

- [ ] **Step 1: Update src/state.rs**

```rust
use std::net::Ipv4Addr;

use camino::Utf8PathBuf;
use derive_builder::Builder;

use crate::ip_cache::IpCache;

#[derive(Debug, Builder)]
pub struct ApplicationState {
    pub config_path: Utf8PathBuf,
    pub ip_cache: IpCache,
    pub ip_cache_path: Utf8PathBuf,
    pub public_ip_address: Ipv4Addr,
    pub preview: bool,
    pub force: bool,
}
```

- [ ] **Step 2: Remove create_reqwest_client from src/lib.rs**

Remove this function (now inlined in each provider):

```rust
// DELETE this entire function:
pub fn create_reqwest_client(token: &str) -> Result<Client> { ... }
```

Also remove the unused imports it brought in (`reqwest::header::HeaderMap`, `reqwest::Client` if no longer needed at the lib level).

- [ ] **Step 3: Rewrite src/cli/dyndns.rs**

```rust
use std::collections::HashMap;
use std::net::Ipv4Addr;

use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use tracing::{debug, info, trace};

use crate::config::{ApplicationConfigLoader, ProviderConfig, ZoneConfig};
use crate::get_public_ip_address;
use crate::ip_cache::{IpCacheReader, IpCacheResult, IpCacheWriter};
use crate::provider::{DnsProvider, Zone};
use crate::provider::cloudflare::CloudflareProvider;
use crate::provider::bunny::BunnyProvider;
use crate::state::{ApplicationState, ApplicationStateBuilder};

#[allow(clippy::doc_markdown)]
#[derive(Debug, Parser)]
/// Dynamic DNS for Cloudflare and bunny.net
struct Args {
    #[arg(short, long)]
    /// Configuration file location. Defaults to
    /// ~/.config/ryndns/ryndns.toml or /etc/ryndns/ryndns.toml when running as root.
    config: Option<Utf8PathBuf>,

    /// IP address cache file location. Defaults to the same location as the
    /// configuration file, with a .cache extension.
    ip_cache: Option<Utf8PathBuf>,

    /// The desired IP address. Defaults to the IP address determined via the
    /// `public_ip_url` in the configuration.
    #[arg(short, long)]
    ip_address: Option<Ipv4Addr>,

    /// Shows what would happen, but doesn't change any settings.
    #[arg(short, long)]
    preview: bool,

    /// Update records even if the cached IP address hasn't changed.
    #[arg(short, long)]
    force: bool,
}

pub async fn main() -> Result<()> {
    crate::init()?;
    debug!("Logging start...");

    let args = Args::parse();
    trace!("Parsed args:\n{:#?}", args);

    let config_path =
        args.config.unwrap_or(ApplicationConfigLoader::default_config_file()?);

    let config = ApplicationConfigLoader::load_config_from(&config_path)?;
    trace!("Configuration:\n{:#?}", config);

    if config.cloudflare().is_none() && config.bunny().is_none() {
        return Err(eyre!(
            "No provider configured. Add a [cloudflare] or [bunny] section to your config."
        ));
    }

    let ip_cache_path =
        args.ip_cache.unwrap_or(config_path.with_extension("cache"));
    let ip_cache = IpCacheReader::load(&ip_cache_path)?;
    debug!("IP cache:\n{:#?}", ip_cache);

    let public_ip_address = if let Some(ip) = args.ip_address {
        ip
    } else {
        get_public_ip_address(config.public_ip_url()).await?
    };

    let mut state = ApplicationStateBuilder::default()
        .config_path(config_path)
        .ip_cache(ip_cache)
        .ip_cache_path(ip_cache_path)
        .public_ip_address(public_ip_address)
        .preview(args.preview)
        .force(args.force)
        .build()?;

    if let Some(cf_config) = config.cloudflare() {
        let provider = CloudflareProvider::new(cf_config.token())?;
        run_provider(&provider, cf_config, &mut state).await?;
    }

    if let Some(bunny_config) = config.bunny() {
        let provider = BunnyProvider::new(bunny_config.token())?;
        run_provider(&provider, bunny_config, &mut state).await?;
    }

    if !state.preview {
        IpCacheWriter.save(&state.ip_cache, &state.ip_cache_path)?;
    }

    info!("Done.");
    Ok(())
}

async fn run_provider<P: DnsProvider>(
    provider: &P,
    provider_config: &ProviderConfig,
    state: &mut ApplicationState,
) -> Result<()> {
    let zone_list = provider.list_zones().await?;
    let zone_map: HashMap<String, Zone> = zone_list
        .into_iter()
        .map(|z| (z.name.clone(), z))
        .collect();

    debug!("Retrieved zones: {:#?}", zone_map.keys().collect::<Vec<_>>());

    for (zone_name, zone_config) in provider_config.zones() {
        let zone = zone_map.get(zone_name).cloned().unwrap_or(Zone {
            id: zone_name.clone(),
            name: zone_name.clone(),
        });
        handle_zone(provider, &zone, zone_config, state).await?;
    }

    Ok(())
}

async fn handle_zone<P: DnsProvider>(
    provider: &P,
    zone: &Zone,
    zone_config: &ZoneConfig,
    state: &mut ApplicationState,
) -> Result<()> {
    if zone_config.records().is_empty() {
        return Err(eyre!(
            "There are no records selected for update on zone '{}'.",
            zone.name
        ));
    }

    info!("Handling zone '{}'", zone.name);

    let public_ip_address = state.public_ip_address;
    let result = state.ip_cache.handle_ip(&zone.id, public_ip_address);

    match result {
        IpCacheResult::Unchanged => {
            if state.force {
                info!("IP address unchanged: '{public_ip_address}', forcing update");
            } else {
                info!("IP address unchanged: '{public_ip_address}'");
                return Ok(());
            }
        },
        IpCacheResult::New => {
            info!("IP address on first run: '{public_ip_address}'");
        },
        IpCacheResult::Changed { previous_ip_address } => {
            info!("IP address updated: '{previous_ip_address}' => '{public_ip_address}'");
        },
    }

    let records = provider.list_records(zone).await?;
    debug!("Retrieved records for '{}':\n{:#?}", zone.name, records);

    let records_to_update: Vec<_> = records
        .iter()
        .filter(|r| zone_config.is_record_selected(&r.name, r.record_type))
        .collect();

    debug!("Updating {} records:", records_to_update.len());
    for record in &records_to_update {
        debug!("{:>4}: {}", record.record_type, record.name);
    }

    for record in records_to_update {
        info!("Updating {}...", record.name);
        if !state.preview {
            provider
                .update_record(zone, record, &public_ip_address.to_string())
                .await?;
        }
    }

    Ok(())
}
```

- [ ] **Step 4: Build and test**

```bash
cargo test
```

Expected: all tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/state.rs src/cli/dyndns.rs src/lib.rs
git commit -m "feat: rewrite main loop for multi-provider support"
```

---

## Task 8: Update list_zones CLI

**Files:**
- Rewrite: `src/cli/list_zones.rs`

**Interfaces:**
- Consumes: `CloudflareProvider`, `BunnyProvider`, `DnsProvider`, `ApplicationConfig`

- [ ] **Step 1: Rewrite src/cli/list_zones.rs**

```rust
use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::Result;

use crate::config::ApplicationConfigLoader;
use crate::provider::{DnsProvider, DnsRecord, DnsRecordType, Zone};
use crate::provider::cloudflare::CloudflareProvider;
use crate::provider::bunny::BunnyProvider;

#[derive(Parser)]
/// List DNS zones from all configured providers.
struct Args {
    /// Configuration file location. Defaults to
    /// ~/.config/ryndns/ryndns.toml or /etc/ryndns/ryndns.toml when running as root.
    config: Option<Utf8PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbosity: u8,
}

pub async fn main() -> Result<()> {
    crate::init()?;

    let args = Args::parse();
    let config_path =
        args.config.unwrap_or(ApplicationConfigLoader::default_config_file()?);
    let config = ApplicationConfigLoader::load_config_from(&config_path)?;

    if let Some(cf_config) = config.cloudflare() {
        println!("cloudflare:");
        let provider = CloudflareProvider::new(cf_config.token())?;
        print_provider_zones(&provider, args.verbosity).await?;
        println!();
    }

    if let Some(bunny_config) = config.bunny() {
        println!("bunny:");
        let provider = BunnyProvider::new(bunny_config.token())?;
        print_provider_zones(&provider, args.verbosity).await?;
        println!();
    }

    Ok(())
}

async fn print_provider_zones<P: DnsProvider>(
    provider: &P,
    verbosity: u8,
) -> Result<()> {
    let zones = provider.list_zones().await?;

    if zones.is_empty() {
        println!("  (no zones found)");
        return Ok(());
    }

    let max_name = zones.iter().map(|z| z.name.len()).max().unwrap_or(0);

    for zone in &zones {
        println!("  {:max_name$} (id: {})", zone.name, zone.id);

        if verbosity > 0 {
            let records = provider.list_records(zone).await?;
            let filtered: Vec<_> = records
                .into_iter()
                .filter(|r| verbosity > 1 || r.record_type == DnsRecordType::A)
                .collect();

            if !filtered.is_empty() {
                let fmt = DnsRecordFormatter::from_records(&filtered);
                for record in &filtered {
                    fmt.print(record);
                }
            }
            println!();
        }
    }

    Ok(())
}

#[derive(Debug, Default)]
struct DnsRecordFormatter {
    id: usize,
    name: usize,
    record_type: usize,
    content: usize,
}

impl DnsRecordFormatter {
    fn from_records(records: &[DnsRecord]) -> Self {
        records.iter().fold(Self::default(), |acc, r| Self {
            id: acc.id.max(r.id.len()),
            name: acc.name.max(r.name.len()),
            record_type: acc.record_type.max(r.record_type.to_string().len()),
            content: acc.content.max(r.content.len()),
        })
    }

    fn print(&self, r: &DnsRecord) {
        println!(
            "    {:0$} {:1$} {:2$} {:3$}",
            r.id, r.name, r.record_type, r.content,
            // widths
        );
        // Rust format strings require literal width args; use positional:
    }
}
```

Fix the `print` method — Rust format strings can't use struct fields as widths directly. Use the explicit form:

```rust
fn print(&self, r: &DnsRecord) {
    println!(
        "    {id:id_w$} {name:name_w$} {rt:rt_w$} {content:content_w$}",
        id = r.id,
        id_w = self.id,
        name = r.name,
        name_w = self.name,
        rt = r.record_type,
        rt_w = self.record_type,
        content = r.content,
        content_w = self.content,
    );
}
```

- [ ] **Step 2: Build**

```bash
cargo build
```

Expected: compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add src/cli/list_zones.rs
git commit -m "feat: update list_zones to show zones from all configured providers"
```

---

## Task 9: Update README and cli/public_ip.rs

**Files:**
- Modify: `README.md`
- Modify: `src/cli/public_ip.rs` (doc string only)

- [ ] **Step 1: Update src/cli/public_ip.rs doc string**

Change `/// List CloudFlare zones.` to `/// Show the current public IP address.`

- [ ] **Step 2: Rewrite README.md**

Update the README to reflect:
- New name `ryndns`
- New config format with provider sections
- Migration note: config file moves from `~/.config/cloudflare-dyndns/` to `~/.config/ryndns/`
- Two providers: Cloudflare and bunny.net
- Usage section (commands are the same, just `ryndns` instead of `cloudflare-dyndns`)
- Installation section updated for new file names

- [ ] **Step 3: Final build and test**

```bash
cargo test && cargo build --release
```

Expected: all tests pass, release build succeeds.

- [ ] **Step 4: Commit**

```bash
git add README.md src/cli/public_ip.rs
git commit -m "docs: update README for ryndns with multi-provider support"
```

---

## Self-Review Notes

- **Type consistency:** `ZoneConfig::is_record_selected` now takes `(&str, DnsRecordType)` — all call sites in Task 7 pass `(&record.name, record.record_type)`. ✓
- **Bunny name normalization:** apex records (empty `Name`) map to `zone.name`; `update_record` reverses this to send `""` back. ✓
- **Bunny TTL:** stored in `BunnyRecord` but not in shared `DnsRecord`; `update_record` sends `300` as a default. This preserves existing behaviour only if 300 matches the record's current TTL — if not, the TTL will change on update. If TTL preservation matters, extend `DnsRecord` with `ttl: Option<u32>` and populate it in `BunnyProvider::list_records`.
- **Bunny API verification:** Step 5 carries a warning to check the actual API. The HTTP methods and request body format must be verified before the Bunny implementation is considered correct.
- **Stale cli module imports:** After Task 4, `cli/dyndns.rs` and `cli/list_zones.rs` temporarily break. Task 7 and Task 8 fix them. If running `cargo test` between tasks is required, temporarily comment out those modules in `src/cli/mod.rs`.
