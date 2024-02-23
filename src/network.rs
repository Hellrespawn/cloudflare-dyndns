use color_eyre::eyre::eyre;
use color_eyre::Result;
use reqwest::header::HeaderMap;
use reqwest::Client;

/// Create client with Content-Type and Authorization headers.
pub fn create_reqwest_client(token: &str) -> Result<Client> {
    let mut headers = HeaderMap::new();

    headers.insert(
        "Content-Type",
        "application/json"
            .parse()
            .map_err(|_| eyre!("Invalid 'Content-Type' header."))?,
    );
    headers.insert(
        "Authorization",
        format!("Bearer {}", token)
            .parse()
            .map_err(|_| eyre!("Invalid 'Authorization' header."))?,
    );

    let client =
        Client::builder().default_headers(headers).use_rustls_tls().build()?;

    Ok(client)
}
