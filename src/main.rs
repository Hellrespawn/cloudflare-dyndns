use cloudflare_dyndns::cli;
use color_eyre::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::main().await
}
