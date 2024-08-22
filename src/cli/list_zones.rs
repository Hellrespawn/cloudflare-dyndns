use color_eyre::Result;

use crate::cloudflare_api::endpoints::list_zones;
use crate::config::Config;
use crate::create_reqwest_client;

pub async fn main() -> Result<()> {
    crate::init()?;

    let config = Config::load_config()?;

    let client = create_reqwest_client(config.cloudflare_token())?;

    let list_zones_response = list_zones(&client).await?;

    let zones = list_zones_response.result;

    if zones.is_empty() {
        println!("Found 0 zones.");
    } else {
        println!("Found {} zones:", zones.len());

        for zone in zones {
            println!("{} ({})", zone.name, zone.id);
        }
    }

    Ok(())
}
