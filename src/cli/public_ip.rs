use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::Result;
use tracing::{debug, info, warn};

use crate::config::env::Env;
use crate::config::ip::IpAddress;

pub async fn main() -> Result<()> {
    crate::init()?;

    let args = Args::parse();

    let env = if let Some(path) = &args.config_file {
        Env::from_file(path)?
    } else {
        None
    };

    let url = args.url.or_else(|| env.map(|e| e.ip_url)?);

    if let Some(url) = url {
        debug!("Querying: {}", url);

        let address = IpAddress::Url(url).ip_address().await?;

        info!("Public IP address: {}", address);
    } else {
        warn!("URL not provided!");
    }

    Ok(())
}

/// Check public or user-supplied IP address.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// User-supplied URL to query public IP-address
    #[arg(long)]
    pub url: Option<String>,

    /// Custom configuration file.
    #[arg(short, long)]
    pub config_file: Option<Utf8PathBuf>,
}

impl Args {
    pub fn parse() -> Args {
        <Args as Parser>::parse()
    }
}
