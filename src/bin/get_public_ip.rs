use cloudflare_dyndns::ip::get_ip_query_from_args;
use cloudflare_dyndns::Args;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let query = get_ip_query_from_args(&args);
    let addr = query.get_public_ip_address().await?;

    println!("Your public ip address is: {}", addr);

    Ok(())
}
