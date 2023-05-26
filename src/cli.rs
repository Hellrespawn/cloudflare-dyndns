use crate::cloudflare::{get_records, patch_record, DNSRecord};
use crate::ip::{
    get_previous_ip_address, get_public_ip_address, update_previous_ip_address,
};
use crate::{Args, Config};
use anyhow::{anyhow, Result};
use reqwest::header::HeaderMap;
use reqwest::Client;

pub async fn main() -> Result<()> {
    let Args { ip_address } = Args::parse();
    let Config { token, zone_id } = Config::read()?;

    let client = create_client(&token)?;

    let new_ip_address = if let Some(ip_address) = ip_address {
        ip_address
    } else {
        get_public_ip_address().await?
    };

    let previous_ip_address = get_previous_ip_address();

    let is_ip_changed =
        is_ip_changed(&new_ip_address, previous_ip_address.as_deref());

    if is_ip_changed {
        let a_records = get_a_records(&client, &zone_id).await?;

        println!("Retrieved DNS records.");

        update_a_records(&client, &zone_id, &a_records, &new_ip_address)
            .await?;

        update_previous_ip_address(&new_ip_address)?;
        println!("Updated IP address in cache.");
    }

    println!("Done.");

    Ok(())
}

/** Create client with Content-Type and Authorization headers. */
fn create_client(token: &str) -> Result<Client> {
    let mut headers = HeaderMap::new();

    headers.insert(
        "Content-Type",
        "application/json"
            .parse()
            .map_err(|_| anyhow!("Invalid 'Content-Type' header."))?,
    );
    headers.insert(
        "Authorization",
        format!("Bearer {}", token)
            .parse()
            .map_err(|_| anyhow!("Invalid 'Authorization' header."))?,
    );

    let client = Client::builder().default_headers(headers).build()?;

    Ok(client)
}

fn is_ip_changed(
    new_ip_address: &str,
    previous_ip_address: Option<&str>,
) -> bool {
    if let Some(previous_ip_address) = previous_ip_address {
        if new_ip_address == previous_ip_address {
            println!("IP address unchanged: '{new_ip_address}'");
            false
        } else {
            println!(
            "IP address updated: '{previous_ip_address}' => '{new_ip_address}'"
        );
            true
        }
    } else {
        println!("IP address: '{new_ip_address}'");
        true
    }
}

async fn get_a_records(
    client: &Client,
    zone_id: &str,
) -> Result<Vec<DNSRecord>> {
    let a_records = get_records(client, zone_id)
        .await?
        .into_iter()
        .filter(|r| r.record_type == "A")
        .collect::<Vec<_>>();

    Ok(a_records)
}

async fn update_a_records(
    client: &Client,
    zone_id: &str,
    a_records: &[DNSRecord],
    new_ip_address: &str,
) -> Result<()> {
    for record in a_records {
        patch_record(client, zone_id, &record.id, new_ip_address).await?;

        println!("Updated '{}' record.", record.name);
    }

    Ok(())
}
