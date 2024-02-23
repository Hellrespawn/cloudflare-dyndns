use std::net::Ipv4Addr;

use cloudflare_dyndns::cloudflare::GetRecordsResponse;
use cloudflare_dyndns::config::{default_settings, Args};
use cloudflare_dyndns::ip::IpAddressChange;
use cloudflare_dyndns::{create_reqwest_client, Settings};
use color_eyre::Result;
use reqwest::Client;
use tracing::{debug, info, warn};
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

const LOG_KEY: &str = "LOG";

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    if (std::env::var_os(LOG_KEY)).is_none() {
        std::env::set_var(LOG_KEY, "info");
    }

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_env("LOG"))
        .init();

    let args = Args::parse();

    let settings = default_settings(args)?;

    let client = create_reqwest_client(&settings.token)?;

    let addr = settings.ip.ip_address().await?;

    let zone_id = settings.zone.get_zone_id(&client).await?;

    let ip_address_change = IpAddressChange::new(addr, &settings)?;

    info!("{ip_address_change}");

    if let Some(new_ip_address) =
        ip_address_change.get_new_ip_address(settings.force)
    {
        if settings.force {
            warn!("Running forced update...");
        }

        update_ip_address(&client, &settings, &zone_id, new_ip_address).await?;
    }

    debug!("Done.");

    Ok(())
}

async fn update_ip_address(
    client: &Client,
    settings: &Settings,
    zone_id: &str,
    new_ip_address: Ipv4Addr,
) -> Result<()> {
    let get_records_response = GetRecordsResponse::get(client, zone_id).await?;

    debug!("Retrieved DNS records.");

    let patch_record_bodies =
        get_records_response.create_patch_record_bodies(new_ip_address);

    for patch_record_body in patch_record_bodies {
        if !settings.preview {
            patch_record_body.patch(client, zone_id).await?;
        }

        info!(
            "{}Updated '{}' record.",
            if settings.preview { "[Preview]: " } else { "" },
            patch_record_body.name
        );
    }

    if !settings.preview {
        IpAddressChange::update_previous_ip_address(new_ip_address, settings)?;
    }

    info!(
        "{}Updated IP address in cache.",
        if settings.preview { "[Preview]: " } else { "" }
    );

    Ok(())
}
