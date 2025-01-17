#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    cloudflare_dyndns::cli::public_ip::main().await
}
