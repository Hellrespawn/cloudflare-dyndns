use color_eyre::Result;
use tabled::settings::peaker::PriorityMax;
use tabled::settings::{Style, Width};
use tabled::Table;

use crate::cloudflare_api::endpoints::{get_records, list_zones};
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
            let records = get_records(&client, &zone.id).await?;

            println!("{} ({})", zone.name, zone.id);

            let (terminal_size::Width(width), _) =
                terminal_size::terminal_size().unwrap();

            let width = width as usize;

            let mut table = Table::new(records.result);

            table.with(Style::empty());

            table.with(Width::truncate(width).suffix("...").priority(PriorityMax));
            table.with(Width::increase(width));

            println!("{table}");

            println!();

            // break;
        }
    }

    Ok(())
}
