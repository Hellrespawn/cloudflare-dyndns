use std::net::Ipv4Addr;

use cloudflare_dyndns::cloudflare::{GetRecordsResponse, ListZonesResponse};
use cloudflare_dyndns::config::ZoneIdentifier;
use cloudflare_dyndns::ip::{get_ip_query_from_args, IpAddress};
use cloudflare_dyndns::{create_reqwest_client, Args, Settings};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use reqwest::Client;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let settings = Settings::read()?;

    let addr = if let Some(addr) = &args.ip_address {
        addr.parse()?
    } else {
        let query = get_ip_query_from_args(&args);
        query.get_public_ip_address().await?
    };

    let client = create_reqwest_client(&settings.token)?;

    let zone_id = get_zone_id(&client, &settings).await?;

    let ip_address = IpAddress::new(addr, &settings)?;

    println!("{ip_address}");

    if let Some(new_ip_address) = ip_address.get_new_ip_address(args.force) {
        if args.force {
            println!("Running forced update...");
        }

        update_ip_address(&client, &args, &settings, &zone_id, new_ip_address)
            .await?;
    }

    println!("Done.");

    Ok(())
}

async fn get_zone_id(client: &Client, settings: &Settings) -> Result<String> {
    let zone = &settings.zone;

    let zone_id = match zone {
        ZoneIdentifier::Id(id) => id.to_owned(),
        ZoneIdentifier::Name(name) => {
            let list_zones_response = ListZonesResponse::get(client).await?;

            let zone_response = list_zones_response
                .find_by_name(name)
                .ok_or(eyre!("Unable to find zone with name '{}'", name))?;

            println!("Updating zone '{}'", name);

            zone_response.id().to_owned()
        },
    };

    Ok(zone_id)
}

async fn update_ip_address(
    client: &Client,
    args: &Args,
    settings: &Settings,
    zone_id: &str,
    new_ip_address: Ipv4Addr,
) -> Result<()> {
    let get_records_response = GetRecordsResponse::get(client, zone_id).await?;

    println!("Retrieved DNS records.");

    let patch_record_bodies =
        get_records_response.create_patch_record_bodies(new_ip_address);

    for patch_record_body in patch_record_bodies {
        if args.preview {
            print!("[Preview]: ");
        } else {
            patch_record_body.patch(client, zone_id).await?;
        }

        println!("Updated '{}' record.", patch_record_body.name);
    }

    if args.preview {
        print!("[Preview]: ");
    } else {
        IpAddress::update_previous_ip_address(new_ip_address, settings)?;
    }

    println!("Updated IP address in cache.");

    Ok(())
}
