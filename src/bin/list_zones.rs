#[tokio::main(flavor = "current_thread")]
async fn main() -> color_eyre::Result<()> {
    ryndns::cli::list_zones::main().await
}
