use camino::Utf8PathBuf;
use clap::Parser;
use color_eyre::Result;

use crate::config::ApplicationConfigLoader;
use crate::get_public_ip_address;

#[derive(Parser)]
/// List `CloudFlare` zones.
struct Args {
    /// Configuration file location. Defaults to
    /// ~/.config/cloudflare-dyndns.toml or
    /// /etc/cloudflare-dyndns/cloudflare-dyndns.toml when running as root.
    config: Option<Utf8PathBuf>,
}

pub async fn main() -> Result<()> {
    crate::init()?;

    let args = Args::parse();

    let config_path =
        args.config.unwrap_or(ApplicationConfigLoader::default_config_file()?);

    let config = ApplicationConfigLoader::load_config_from(&config_path)?;

    let ip_url = config.public_ip_url();

    let public_ip = get_public_ip_address(ip_url).await?;

    println!("Your public IP address is {public_ip}");

    Ok(())
}
