use color_eyre::Result;
use reqwest::Client;

use crate::cloudflare_api::record::{get_records, DNSRecord, DNSRecordType};
use crate::cloudflare_api::zone::{list_zones, ZoneResponse};
use crate::config::Config;
use crate::create_reqwest_client;

pub async fn main() -> Result<()> {
    crate::init()?;

    let config = Config::load_config()?;

    let verbose = is_verbose();

    let client = create_reqwest_client(config.cloudflare_token())?;

    let zones = list_zones(&client).await?;

    if zones.is_empty() {
        println!("Found 0 zones.");
    } else {
        print_zones(&client, &zones, verbose).await?;
    }

    Ok(())
}

fn is_verbose() -> bool {
    let args = std::env::args();

    if args.len() > 1 {
        for arg in args {
            if matches!(arg.as_str(), "-v" | "--verbose") {
                return true;
            }
        }
    }

    false
}

async fn print_zones(
    client: &Client,
    zones: &[ZoneResponse],
    verbose: bool,
) -> Result<()> {
    let max_name_length = zones.iter().map(|z| z.name.len()).max().unwrap_or(0);

    println!("Found {} zones:", zones.len());

    for zone in zones {
        print_zone(client, zone, verbose, max_name_length).await?;
    }

    Ok(())
}

#[derive(Debug, Default)]
struct DNSRecordFormatter {
    id: usize,
    name: usize,
    record_type: usize,
    content: usize,
}

impl DNSRecordFormatter {
    fn from_records(records: &[DNSRecord]) -> Self {
        let mut alignment = DNSRecordFormatter::default();

        for record in records {
            alignment = alignment.add(record);
        }

        alignment
    }

    fn add(self, record: &DNSRecord) -> Self {
        Self {
            id: std::cmp::max(record.id.len(), self.id),
            name: std::cmp::max(record.name.len(), self.name),
            record_type: std::cmp::max(
                record.record_type.to_string().len(),
                self.record_type,
            ),
            content: std::cmp::max(record.content.len(), self.content),
        }
    }

    // fn width(&self) -> usize {
    //     // Add spaces between columns
    //     self.id + self.name + self.record_type + self.content + 3
    // }

    fn print(&self, DNSRecord { id, name, record_type, content }: &DNSRecord) {
        println!(
            "{id:0$} {name:1$} {record_type:2$} {content:3$}",
            self.id, self.name, self.record_type, self.content
        );
    }
}

async fn print_zone(
    client: &Client,
    zone: &ZoneResponse,
    verbose: bool,
    max_name_length: usize,
) -> Result<()> {
    println!("{:max_name_length$} ({})", zone.name, zone.id);

    if verbose {
        let records = get_records(client, &zone.id)
            .await?
            .into_iter()
            .filter(|record| record.record_type == DNSRecordType::A)
            .collect::<Vec<_>>();

        let alignment = DNSRecordFormatter::from_records(&records);

        for record in records {
            alignment.print(&record);
        }

        println!();
    }

    Ok(())
}
