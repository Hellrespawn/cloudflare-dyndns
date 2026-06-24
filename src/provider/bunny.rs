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

// Bunny stores record names as bare subdomains ("www") and uses "" or "@" for the zone apex.
// We normalise to FQDNs on the way in and reverse on the way out so callers see the same
// name format as Cloudflare.

fn normalize_name(bunny_name: &str, zone_name: &str) -> String {
    if bunny_name.is_empty() || bunny_name == "@" {
        zone_name.to_owned()
    } else {
        format!("{bunny_name}.{zone_name}")
    }
}

fn raw_subdomain<'a>(record_name: &'a str, zone_name: &str) -> &'a str {
    if record_name == zone_name {
        ""
    } else {
        record_name
            .strip_suffix(&format!(".{zone_name}"))
            .unwrap_or(record_name)
    }
}

fn map_bunny_record(r: BunnyRecord, zone_name: &str) -> DnsRecord {
    DnsRecord {
        id: r.id.to_string(),
        name: normalize_name(&r.name, zone_name),
        record_type: DnsRecordType::from(r.record_type),
        content: r.value,
        ttl: Some(r.ttl),
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
        let ttl = record.ttl.unwrap_or(300);
        update_record(
            &self.client,
            zone_id,
            record_id,
            bunny_type_to_u8(record.record_type),
            raw_name,
            ttl,
            new_ip,
        )
        .await
    }
}
