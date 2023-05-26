use anyhow::{bail, Result};
use fs_err as fs;
use std::path::Path;

const PREVIOUS_IP_FILE: &str = "last_ip";

pub async fn get_public_ip_address() -> Result<String> {
    Ok(reqwest::get("https://ipecho.net/plain")
        .await?
        .text()
        .await?)
}

pub fn get_previous_ip_address() -> Result<Option<String>> {
    todo!()
}

fn read_previous_ip_address_from_file<P: AsRef<Path>>(
    path: P,
) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(string) => Ok(Some(string)),
        Err(error) => match error.kind() {
            std::io::ErrorKind::NotFound => Ok(None),
            _ => bail!(error),
        },
    }
}

pub fn update_previous_ip_address(ip_address: &str) -> Result<()> {
    todo!()
}
