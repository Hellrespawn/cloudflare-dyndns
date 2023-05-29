use crate::{read_file_optional, CONFIG_PATHS};
use anyhow::{bail, Result};
use fs_err as fs;

pub enum IpAddress {
    New(String),
    Unchanged(String),
    Changed {
        new_ip_address: String,
        previous_ip_address: String,
    },
}

impl IpAddress {
    pub async fn new(new_ip_address: Option<String>) -> Result<IpAddress> {
        let new_ip_address = if let Some(ip_address) = new_ip_address {
            ip_address
        } else {
            Self::get_public_ip_address().await?
        };

        let previous_ip_address = Self::get_previous_ip_address();

        Ok(Self::get_variant(new_ip_address, previous_ip_address))
    }

    pub fn get_new_ip_address(&self, force: bool) -> Option<&str> {
        match self {
            IpAddress::Changed { new_ip_address, .. }
            | IpAddress::New(new_ip_address) => Some(new_ip_address),
            IpAddress::Unchanged(previous_ip_address) => {
                if force {
                    Some(previous_ip_address)
                } else {
                    None
                }
            }
        }
    }

    pub fn update_previous_ip_address(ip_address: &str) -> Result<()> {
        if fs::write(&CONFIG_PATHS.system.previous_ip, ip_address).is_err()
            && fs::write(&CONFIG_PATHS.user.previous_ip, ip_address).is_err()
        {
            bail!("Unable to update previous IP address: '{ip_address}'")
        }

        Ok(())
    }

    fn get_variant(
        new_ip_address: String,
        previous_ip_address: Option<String>,
    ) -> Self {
        if let Some(previous_ip_address) = previous_ip_address {
            if new_ip_address == previous_ip_address {
                Self::Unchanged(new_ip_address)
            } else {
                Self::Changed {
                    new_ip_address,
                    previous_ip_address,
                }
            }
        } else {
            Self::New(new_ip_address)
        }
    }

    async fn get_public_ip_address() -> Result<String> {
        Ok(reqwest::get("https://ipecho.net/plain")
            .await?
            .text()
            .await?)
    }

    fn get_previous_ip_address() -> Option<String> {
        read_file_optional(&CONFIG_PATHS.user.previous_ip)
            .or_else(|| read_file_optional(&CONFIG_PATHS.system.previous_ip))
    }
}

impl std::fmt::Display for IpAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpAddress::New(new_ip_address) => write!(f, "IP address: '{new_ip_address}'"),
            IpAddress::Unchanged(previous_ip_address) => write!(f, "IP address unchanged: '{previous_ip_address}'"),
            IpAddress::Changed { new_ip_address, previous_ip_address } => write!(f,
                "IP address updated: '{previous_ip_address}' => '{new_ip_address}'"
            ),
        }
    }
}
