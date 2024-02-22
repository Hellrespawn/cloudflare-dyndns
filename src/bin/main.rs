use std::net::Ipv4Addr;

use cloudflare_dyndns::cloudflare::GetRecordsResponse;
use cloudflare_dyndns::config::{default_settings, Args};
use cloudflare_dyndns::ip::IpAddress;
use cloudflare_dyndns::{create_reqwest_client, Settings};
use color_eyre::Result;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let settings = default_settings(args)?;

    let client = create_reqwest_client(&settings.token)?;

    let addr = settings.ip.ip_address().await?;

    let zone_id = settings.zone.get_zone_id(&client).await?;

    let ip_address = IpAddress::new(addr, &settings)?;

    println!("{ip_address}");

    if let Some(new_ip_address) = ip_address.get_new_ip_address(settings.force)
    {
        if settings.force {
            println!("Running forced update...");
        }

        update_ip_address(&client, &settings, &zone_id, new_ip_address).await?;
    }

    println!("Done.");

    Ok(())
}

async fn update_ip_address(
    client: &Client,
    settings: &Settings,
    zone_id: &str,
    new_ip_address: Ipv4Addr,
) -> Result<()> {
    let get_records_response = GetRecordsResponse::get(client, zone_id).await?;

    println!("Retrieved DNS records.");

    let patch_record_bodies =
        get_records_response.create_patch_record_bodies(new_ip_address);

    for patch_record_body in patch_record_bodies {
        if settings.preview {
            print!("[Preview]: ");
        } else {
            patch_record_body.patch(client, zone_id).await?;
        }

        println!("Updated '{}' record.", patch_record_body.name);
    }

    if settings.preview {
        print!("[Preview]: ");
    } else {
        IpAddress::update_previous_ip_address(new_ip_address, settings)?;
    }

    println!("Updated IP address in cache.");

    Ok(())
}
