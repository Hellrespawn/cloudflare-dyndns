use anyhow::Result;
use cloudflare_dyndns::cli;

#[tokio::main]
async fn main() -> Result<()> {
    cli::main().await
}
