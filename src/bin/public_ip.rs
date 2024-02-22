use cloudflare_dyndns::config::{default_settings, Args};
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();
    let settings = default_settings(args)?;

    let addr = settings.ip.ip_address().await?;

    println!("Your public ip address is: {}", addr);

    Ok(())
}
