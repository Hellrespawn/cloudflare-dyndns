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
                ttl: None,
            })
            .collect())
    }

    async fn update_record(
        &self,
        zone: &Zone,
        record: &DnsRecord,
        new_ip: &str,
    ) -> Result<()> {
        patch_record(&self.client, &zone.id, &record.id, new_ip).await
    }
}
