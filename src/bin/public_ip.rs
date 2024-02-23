#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    cloudflare_dyndns::cli::public_ip::main().await
}
