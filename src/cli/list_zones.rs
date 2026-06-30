use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::Result;

use crate::config::ApplicationConfigLoader;
use crate::provider::bunny::BunnyProvider;
use crate::provider::cloudflare::CloudflareProvider;
use crate::provider::{DnsProvider, DnsRecord, DnsRecordType};

#[derive(Parser)]
/// List DNS zones from all configured providers.
struct Args {
    /// Configuration file location. Defaults to
    /// ~/.config/ryndns/ryndns.toml or /etc/ryndns/ryndns.toml when running as root.
    config: Option<Utf8PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbosity: u8,
}

pub async fn main() -> Result<()> {
    crate::init()?;

    let args = Args::parse();
    let config_path =
        args.config.unwrap_or(ApplicationConfigLoader::default_config_file()?);
    let config = ApplicationConfigLoader::load_config_from(&config_path)?;

    if let Some(cf_config) = config.cloudflare() {
        println!("cloudflare:");
        let provider = CloudflareProvider::new(cf_config.token())?;
        print_provider_zones(&provider, args.verbosity).await?;
        println!();
    }

    if let Some(bunny_config) = config.bunny() {
        println!("bunny:");
        let provider = BunnyProvider::new(bunny_config.token())?;
        print_provider_zones(&provider, args.verbosity).await?;
        println!();
    }

    Ok(())
}

async fn print_provider_zones<P: DnsProvider>(
    provider: &P,
    verbosity: u8,
) -> Result<()> {
    let zones = provider.list_zones().await?;

    if zones.is_empty() {
        println!("  (no zones found)");
        return Ok(());
    }

    let max_name = zones.iter().map(|z| z.name.len()).max().unwrap_or(0);

    for zone in &zones {
        println!("  {:max_name$} (id: {})", zone.name, zone.id);

        if verbosity > 0 {
            let records = provider.list_records(zone).await?;
            let filtered: Vec<_> = records
                .into_iter()
                .filter(|r| verbosity > 1 || r.record_type == DnsRecordType::A)
                .collect();

            if !filtered.is_empty() {
                let fmt = DnsRecordFormatter::from_records(&filtered);
                for record in &filtered {
                    fmt.print(record);
                }
            }
            println!();
        }
    }

    Ok(())
}

#[derive(Debug, Default)]
struct DnsRecordFormatter {
    id: usize,
    name: usize,
    record_type: usize,
    content: usize,
}

impl DnsRecordFormatter {
    fn from_records(records: &[DnsRecord]) -> Self {
        records.iter().fold(Self::default(), |acc, r| {
            Self {
                id: acc.id.max(r.id.len()),
                name: acc.name.max(r.name.len()),
                record_type: acc
                    .record_type
                    .max(r.record_type.to_string().len()),
                content: acc.content.max(r.content.len()),
            }
        })
    }

    fn print(&self, r: &DnsRecord) {
        println!(
            "    {id:id_w$} {name:name_w$} {rt:rt_w$} {content:content_w$}",
            id = r.id,
            id_w = self.id,
            name = r.name,
            name_w = self.name,
            rt = r.record_type,
            rt_w = self.record_type,
            content = r.content,
            content_w = self.content,
        );
    }
}
