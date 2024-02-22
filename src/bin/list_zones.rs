use cloudflare_dyndns::cloudflare::ListZonesResponse;
use cloudflare_dyndns::config::{default_settings, Args};
use cloudflare_dyndns::create_reqwest_client;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let settings = default_settings(args)?;

    let client = create_reqwest_client(&settings.token)?;

    let response = ListZonesResponse::get(&client).await?;

    println!("{:#?}", response);

    Ok(())
}
