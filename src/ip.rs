use crate::{
    read_file_optional, SYSTEM_PREVIOUS_IP_PATH, USER_PREVIOUS_IP_PATH,
};
use anyhow::{bail, Result};
use fs_err as fs;

pub async fn get_public_ip_address() -> Result<String> {
    Ok(reqwest::get("https://ipecho.net/plain")
        .await?
        .text()
        .await?)
}

pub fn get_previous_ip_address() -> Option<String> {
    read_file_optional(&USER_PREVIOUS_IP_PATH)
        .or_else(|| read_file_optional(&SYSTEM_PREVIOUS_IP_PATH))
}

pub fn update_previous_ip_address(ip_address: &str) -> Result<()> {
    if fs::write(&*SYSTEM_PREVIOUS_IP_PATH, ip_address).is_err()
        && fs::write(&*USER_PREVIOUS_IP_PATH, ip_address).is_err()
    {
        bail!("Unable to update previous IP address: '{ip_address}'")
    }

    Ok(())
}
