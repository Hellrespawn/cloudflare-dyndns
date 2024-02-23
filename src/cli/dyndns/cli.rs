use std::net::Ipv4Addr;

use color_eyre::Result;
use reqwest::Client;
use tracing::{debug, info, warn};

use crate::cli::dyndns::args::Args;
use crate::cli::dyndns::settings::Settings;
use crate::cloudflare::GetRecordsResponse;
use crate::ip::IpAddressChange;
use crate::network::create_reqwest_client;

pub async fn main() -> Result<()> {
    crate::init()?;

    let args = Args::parse();
    let settings = Settings::default_from_args(args)?;

    let addr = settings.ip.ip_address().await?;

    let client = create_reqwest_client(&settings.token)?;
    let zone_id = settings.zone.get_zone_id(&client).await?;

    let ip_address_change =
        IpAddressChange::new(addr, &settings.get_previous_ip_file())?;

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
        IpAddressChange::write_new_ip_address_to_file(
            new_ip_address,
            &settings.get_previous_ip_file(),
        )?;
    }

    info!(
        "{}Updated IP address in cache.",
        if settings.preview { "[Preview]: " } else { "" }
    );

    Ok(())
}
