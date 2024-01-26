use cloudflare_dyndns::cloudflare::ListZonesResponse;
use cloudflare_dyndns::{create_reqwest_client, Settings};
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let settings = Settings::read()?;

    let client = create_reqwest_client(&settings.token)?;

    let response = ListZonesResponse::get(&client).await?;

    println!("{:#?}", response);

    Ok(())
}
