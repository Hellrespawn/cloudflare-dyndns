pub mod record;
pub mod zone;

const BUNNY_API_URL: &str = "https://api.bunny.net";

pub fn build_bunny_client(token: &str) -> color_eyre::Result<reqwest::Client> {
    use color_eyre::eyre::eyre;
    use reqwest::header::HeaderMap;

    let mut headers = HeaderMap::new();
    headers.insert(
        "Content-Type",
        "application/json"
            .parse()
            .map_err(|_| eyre!("Invalid Content-Type header"))?,
    );
    headers.insert(
        "AccessKey",
        token
            .parse()
            .map_err(|_| eyre!("Invalid AccessKey header"))?,
    );
    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .use_rustls_tls()
        .build()?)
}
