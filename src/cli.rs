use crate::cloudflare::GetRecordsResponse;
use crate::ip::IpAddress;
use crate::{Args, Config};
use anyhow::{anyhow, Result};
use reqwest::header::HeaderMap;
use reqwest::Client;

pub async fn main() -> Result<()> {
    let Args { ip_address } = Args::parse();
    let Config { token, zone_id } = Config::read()?;

    let client = create_client(&token)?;

    let ip_address = IpAddress::new(ip_address).await?;

    println!("{ip_address}");

    if let Some(new_ip_address) = ip_address.get_new_ip_address() {
        let get_records_response =
            GetRecordsResponse::get(&client, &zone_id).await?;

        println!("Retrieved DNS records.");

        let patch_record_bodies =
            get_records_response.create_patch_record_bodies(new_ip_address);

        for patch_record_body in patch_record_bodies {
            patch_record_body.patch(&client, &zone_id).await?;
            println!("Updated '{}' record.", patch_record_body.name);
        }

        IpAddress::update_previous_ip_address(new_ip_address)?;
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
