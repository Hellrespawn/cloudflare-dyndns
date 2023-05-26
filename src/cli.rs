use anyhow::Result;
use reqwest::header::HeaderMap;
use reqwest::Client;

use crate::cloudflare::{get_records, patch_record};
use crate::ip::{
    get_previous_ip_address, get_public_ip_address, update_previous_ip_address,
};
use crate::{Args, Config};

pub async fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::read()?;

    let mut headers = HeaderMap::new();

    headers.insert(
        "Content-Type",
        "application/json"
            .parse()
            .expect("Invalid 'Content-Type' header."),
    );
    headers.insert(
        "Authorization",
        format!("Bearer {}", &config.token)
            .parse()
            .expect("Invalid 'Authorization' header."),
    );

    let client = Client::builder().default_headers(headers).build()?;

    let new_ip_addres = if let Some(ip_address) = args.ip_address {
        ip_address
    } else {
        get_public_ip_address().await?
    };

    let previous_ip_address = get_previous_ip_address()?;

    let is_ip_changed = if let Some(previous_ip_address) = previous_ip_address {
        if new_ip_addres == previous_ip_address {
            println!("IP address unchanged: '{new_ip_addres}'");
            false
        } else {
            println!(
                "IP address updated: '{previous_ip_address}' => '{new_ip_addres}'"
            );
            true
        }
    } else {
        println!("IP address: '{new_ip_addres}'");
        true
    };

    if is_ip_changed {
        let a_records = get_records(&client, &config.zone_id)
            .await?
            .into_iter()
            .filter(|r| r.record_type == "A")
            .collect::<Vec<_>>();

        println!("Retrieved DNS records.");

        for record in a_records {
            patch_record(&client, &config.zone_id, &record.id, &new_ip_addres)
                .await?;

            println!("Updated '{}' record.", record.name);
        }

        update_previous_ip_address(&new_ip_addres)?;
        println!("Updated IP address in cache.");
    }

    Ok(())
}
