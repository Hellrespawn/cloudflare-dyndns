use std::collections::HashMap;
use std::net::Ipv4Addr;

use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::eyre::{eyre, Report};
use color_eyre::Result;
use reqwest::Client;
use tokio::sync::OnceCell;
use tracing::{debug, info};

use crate::cloudflare_api::record::{get_records, patch_record};
use crate::cloudflare_api::zone::list_zones;
use crate::config::{Config, ZoneConfig};
use crate::public_ip::get_public_ip_address;
use crate::public_ip::ip_cache::{IpCache, IpCacheResult};

#[allow(clippy::doc_markdown)]
#[derive(Parser)]
/// Dynamic DNS for CloudFlare
struct Args {
    #[arg(short, long)]
    /// Config file location. Defaults to ~/.config/cloudflare-dyndns.toml or
    /// /etc/cloudflare-dyndns/cloudflare-dyndns.toml when running as root.
    config: Option<Utf8PathBuf>,

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

struct Options {
    client: Client,
    ip_cache: IpCache,
    public_ip_address: Ipv4Addr,
    preview: bool,
    force: bool,
}

impl Options {
    async fn new(args: &Args, config: &Config) -> Result<Self> {
        let ip_cache = IpCache::load(config.cache_file())?;

        let public_ip_address = if let Some(ip_address) = args.ip_address {
            ip_address
        } else {
            get_public_ip_address(config.public_ip_url()).await?
        };

        let client = crate::create_reqwest_client(config.cloudflare_token())?;

        Ok(Options {
            client,
            ip_cache,
            public_ip_address,
            preview: args.preview,
            force: args.force,
        })
    }
}

static ZONE_NAME_TO_ID_MAP: OnceCell<HashMap<String, String>> =
    OnceCell::const_new();

pub async fn main() -> Result<()> {
    crate::init()?;

    let args = Args::parse();

    let config = if let Some(config_path) = &args.config {
        Config::load_config_from(config_path)?
    } else {
        Config::load_config()?
    };

    let mut options = Options::new(&args, &config).await?;

    for (zone, config) in config.zones() {
        handle_zone(&mut options, zone, config).await?;
    }

    if !options.preview {
        options.ip_cache.save()?;
    }

    info!("Done.");

    Ok(())
}

async fn get_zone_name_to_id_map(
    client: &Client,
) -> Result<&HashMap<String, String>> {
    ZONE_NAME_TO_ID_MAP
        .get_or_try_init(|| {
            async {
                let list_zones_response = list_zones(client).await?;

                let map = list_zones_response
                    .into_iter()
                    .map(|z| (z.name, z.id))
                    .collect::<HashMap<_, _>>();

                debug!("Retrieved zones: {map:#?}");

                Ok::<_, Report>(map)
            }
        })
        .await
}

async fn handle_zone(
    options: &mut Options,
    zone_name_or_id: &str,
    zone_config: &ZoneConfig,
) -> Result<()> {
    if zone_config.records().is_empty() {
        Err(eyre!("There are no records selected for update on zone '{zone_name_or_id}'."))
    } else {
        info!("Handling zone '{}'", zone_name_or_id);

        let zone_to_id_map = get_zone_name_to_id_map(&options.client).await?;

        let zone_id = zone_to_id_map
            .get(zone_name_or_id)
            .map_or(zone_name_or_id, |s| s.as_str());

        let public_ip_address = options.public_ip_address;

        let result = options.ip_cache.handle_ip(zone_id, public_ip_address);

        match result {
            IpCacheResult::Unchanged => {
                if options.force {
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

        let records = get_records(&options.client, zone_id).await?;

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
            if !options.preview {
                let patch_record_response = patch_record(
                    &options.client,
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
