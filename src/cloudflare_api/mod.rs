use color_eyre::eyre::{eyre, Error};
use serde::Deserialize;

pub mod record;
pub mod zone;

const API_URL: &str = "https://api.cloudflare.com/client/v4";

fn transform_error_responses(errors: &[CloudFlareError]) -> Error {
    eyre!(errors
        .iter()
        .map(|e| format!("{}: {}", e.code, e.message))
        .collect::<Vec<_>>()
        .join("\n"))
}

#[derive(Deserialize, Debug)]
struct CloudFlareError {
    pub code: isize,
    pub message: String,
}
