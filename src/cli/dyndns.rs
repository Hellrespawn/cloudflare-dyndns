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

    for zone_config in provider_config.zones() {
        let zone = zone_map.get(&zone_config.name).cloned().unwrap_or_else(|| {
            tracing::warn!(
                "Zone '{}' not found in provider's zone list — treating as zone ID directly",
                zone_config.name
            );
            Zone { id: zone_config.name.clone(), name: zone_config.name.clone() }
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
