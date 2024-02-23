use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::Result;
use tracing::{info, warn};

use crate::cloudflare::ListZonesResponse;
use crate::config::env::Env;
use crate::network::create_reqwest_client;

pub async fn main() -> Result<()> {
    crate::init()?;

    let args = Args::parse();

    let env = if let Some(path) = &args.config_file {
        Env::from_file(path)?
    } else {
        None
    };

    let token = args.token.or_else(|| env.map(|e| e.token)?);

    if let Some(token) = token {
        let client = create_reqwest_client(&token)?;

        let list_zones_response = ListZonesResponse::get(&client).await?;

        let zones = list_zones_response.zones();

        info!("Found {} zones.", zones.len());

        for zone in zones {
            info!("{} ({})", zone.name(), zone.id());
        }
    } else {
        warn!("Zone ID not provided!");
    }

    Ok(())
}

/// List available Cloudflare zones.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Cloudflare DNS:Edit token
    #[arg(short, long)]
    pub token: Option<String>,

    /// Custom configuration file.
    #[arg(short, long)]
    pub config_file: Option<Utf8PathBuf>,
}

impl Args {
    #[must_use]
    pub fn parse() -> Args {
        <Args as Parser>::parse()
    }
}
