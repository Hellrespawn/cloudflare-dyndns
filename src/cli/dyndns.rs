use std::collections::HashMap;
use std::net::Ipv4Addr;

use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::Result;
use color_eyre::eyre::eyre;
use reqwest::Client;
use tracing::{debug, info};

use crate::cloudflare_api::record::{get_records, patch_record};
use crate::cloudflare_api::zone::list_zones;
use crate::config::{ApplicationConfigLoader, ZoneConfig};
use crate::get_public_ip_address;
use crate::ip_cache::{IpCacheReader, IpCacheResult, IpCacheWriter};
use crate::state::{ApplicationState, ApplicationStateBuilder};

#[allow(clippy::doc_markdown)]
#[derive(Parser)]
/// Dynamic DNS for CloudFlare
struct Args {
    #[arg(short, long)]
    /// Configuration file location. Defaults to
    /// ~/.config/cloudflare-dyndns.toml or
    /// /etc/cloudflare-dyndns/cloudflare-dyndns.toml when running as root.
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

    let args = Args::parse();

    let config_path =
        args.config.unwrap_or(ApplicationConfigLoader::default_config_file()?);

    let config = ApplicationConfigLoader::load_config_from(&config_path)?;

    let ip_cache_path =
        args.ip_cache.unwrap_or(config_path.with_extension("cache"));

    let ip_cache = IpCacheReader::load(&ip_cache_path)?;

    let client = crate::create_reqwest_client(config.cloudflare_token())?;

    let public_ip_address = if let Some(ip_address) = args.ip_address {
        ip_address
    } else {
        get_public_ip_address(config.public_ip_url()).await?
    };

    let mut state = ApplicationStateBuilder::default()
        .client(client)
        .config_path(config_path)
        .ip_cache(ip_cache)
        .ip_cache_path(ip_cache_path)
        .public_ip_address(public_ip_address)
        .preview(args.preview)
        .force(args.force)
        .build()?;

    for (zone, zone_config) in config.zones() {
        handle_zone(&mut state, zone, zone_config).await?;
    }

    if !state.preview {
        IpCacheWriter.save(&state.ip_cache, &state.ip_cache_path)?;
    }

    info!("Done.");

    Ok(())
}

async fn get_zone_name_to_id_map(
    client: &Client,
) -> Result<HashMap<String, String>> {
    let list_zones_response = list_zones(client).await?;

    let map = list_zones_response
        .into_iter()
        .map(|z| (z.name, z.id))
        .collect::<HashMap<_, _>>();

    debug!("Retrieved zones: {map:#?}");

    Ok(map)
}

async fn handle_zone(
    state: &mut ApplicationState,
    zone_name_or_id: &str,
    zone_config: &ZoneConfig,
) -> Result<()> {
    if zone_config.records().is_empty() {
        Err(eyre!(
            "There are no records selected for update on zone '{zone_name_or_id}'."
        ))
    } else {
        info!("Handling zone '{}'", zone_name_or_id);

        let zone_to_id_map = get_zone_name_to_id_map(&state.client).await?;

        let zone_id = zone_to_id_map
            .get(zone_name_or_id)
            .map_or(zone_name_or_id, |s| s.as_str());

        let public_ip_address = state.public_ip_address;

        let result = state.ip_cache.handle_ip(zone_id, public_ip_address);

        match result {
            IpCacheResult::Unchanged => {
                if state.force {
                    info!(
                        "IP address unchanged: '{public_ip_address}', forcing update"
                    );
                } else {
                    info!("IP address unchanged: '{public_ip_address}'");
                    return Ok(());
                }
            },
            IpCacheResult::New => {
                info!("IP address on first run: '{public_ip_address}'");
            },
            IpCacheResult::Changed { previous_ip_address } => {
                info!(
                    "IP address updated: '{previous_ip_address}' => '{public_ip_address}'"
                );
            },
        }

        let records = get_records(&state.client, zone_id).await?;

        debug!("Retrieved records for '{zone_name_or_id}':\n{records:#?}");

        let records_to_update = records
            .iter()
            .filter(|r| zone_config.is_record_selected(r))
            .collect::<Vec<_>>();

        debug!("Updating {} records:", records_to_update.len());

        for record in &records_to_update {
            debug!("{:>4}: {}", record.record_type, record.name);
        }

        for record in records_to_update {
            info!("Updating {}...", record.name);
            if !state.preview {
                let patch_record_response = patch_record(
                    &state.client,
                    zone_id,
                    &record.id,
                    &public_ip_address.to_string(),
                )
                .await;

                patch_record_response?;
            }
        }

        Ok(())
    }
}
