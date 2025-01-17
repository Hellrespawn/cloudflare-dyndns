#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    cloudflare_dyndns::cli::list_zones::main().await
}
