use std::collections::HashMap;
use std::net::Ipv4Addr;

use color_eyre::eyre::{eyre, Report};
use color_eyre::Result;
use reqwest::Client;
use tokio::sync::OnceCell;
use tracing::{debug, info};

use crate::cli::dyndns::ip_cache::{IpCache, IpCacheResult};
use crate::cloudflare_api::endpoints::{get_records, list_zones, patch_record};
use crate::cloudflare_api::{DNSRecordResponse, PatchRecordRequest};
use crate::config::{Config, ZoneConfig};
use crate::public_ip::get_public_ip_address;

static ZONE_NAME_TO_ID_MAP: OnceCell<HashMap<String, String>> =
    OnceCell::const_new();

pub async fn main() -> Result<()> {
    crate::init()?;

    let config = Config::load_config()?;

    let mut cache = IpCache::load(config.cache_file())?.unwrap_or_default();

    let public_ip_address =
        get_public_ip_address(config.public_ip_url()).await?;

    info!("Public ip address is {public_ip_address}");

    let client = crate::create_reqwest_client(config.cloudflare_token())?;

    for (zone, config) in config.zones() {
        handle_zone(&client, zone, config, &mut cache, public_ip_address)
            .await?;
    }

    cache.save(config.cache_file())?;

    info!("Done.");

    Ok(())
}

async fn get_zone_name_to_id_map(
    client: &Client,
) -> Result<&HashMap<String, String>> {
    ZONE_NAME_TO_ID_MAP
        .get_or_try_init(|| async {
            let list_zones_response = list_zones(client).await?;

            let map = list_zones_response
                .result
                .into_iter()
                .map(|z| (z.name, z.id))
                .collect::<HashMap<_, _>>();

            debug!("Retrieved zones: {map:#?}");

            Ok::<_, Report>(map)
        })
        .await
}

async fn handle_zone(
    client: &Client,
    zone_name_or_id: &str,
    zone_config: &ZoneConfig,
    ip_cache: &mut IpCache,
    public_ip_address: Ipv4Addr,
) -> Result<()> {
    if zone_config.records().is_empty() {
        Err(eyre!("There are no records selected for update on zone '{zone_name_or_id}'."))
    } else {
        let zone_to_id_map = get_zone_name_to_id_map(client).await?;

        let zone_id = zone_to_id_map
            .get(zone_name_or_id)
            .map_or(zone_name_or_id, |s| s.as_str());

        let result = ip_cache.handle_ip(zone_id, public_ip_address);

        match result {
            IpCacheResult::Unchanged => {
                info!("IP address unchanged: '{public_ip_address}'");
                return Ok(());
            },
            IpCacheResult::New => {
                info!("IP address on first run: '{public_ip_address}'");
            },
            IpCacheResult::Changed { previous_ip_address } => {
                info!("IP address updated: '{previous_ip_address}' => '{public_ip_address}'");
            },
        }

        let records = get_records(client, zone_id).await?.result;

        debug!("Retrieved records for '{zone_name_or_id}':\n{records:#?}");

        let records_to_update = records
            .iter()
            .filter(|r| is_record_selected(r, zone_config))
            .collect::<Vec<_>>();

        debug!("Updating {} records:", records_to_update.len());

        for record in &records_to_update {
            debug!("{:>4}: {}", record.record_type, record.name);
        }

        let requests = records_to_update
            .into_iter()
            .map(|r| {
                (
                    r,
                    PatchRecordRequest {
                        content: public_ip_address.to_string(),
                    },
                )
            })
            .collect::<Vec<_>>();

        for (record, request) in requests {
            info!("Updating {}...", record.name);
            let patch_record_response =
                patch_record(client, zone_id, &record.id, request).await;

            patch_record_response?;
        }

        Ok(())
    }
}

fn is_record_selected(
    record_response: &DNSRecordResponse,
    zone_config: &ZoneConfig,
) -> bool {
    let DNSRecordResponse { name, record_type, .. } = record_response;

    zone_config.records().iter().any(|r| r.is_match(name, record_type))
}
