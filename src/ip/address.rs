use std::net::Ipv4Addr;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use fs_err as fs;

use crate::{read_file_optional, Settings};

pub enum IpAddress {
    New(Ipv4Addr),
    Unchanged(Ipv4Addr),
    Changed { new_ip_address: Ipv4Addr, previous_ip_address: Ipv4Addr },
}

impl IpAddress {
    pub fn new(
        new_ip_address: Ipv4Addr,
        settings: &Settings,
    ) -> Result<IpAddress> {
        let previous_ip_address = Self::get_previous_ip_address(settings)?;

        Ok(Self::determine_variant(new_ip_address, previous_ip_address))
    }

    pub fn get_new_ip_address(&self, force: bool) -> Option<Ipv4Addr> {
        match self {
            IpAddress::Changed { new_ip_address, .. }
            | IpAddress::New(new_ip_address) => Some(*new_ip_address),
            IpAddress::Unchanged(previous_ip_address) => {
                if force {
                    Some(*previous_ip_address)
                } else {
                    None
                }
            },
        }
    }

    pub fn update_previous_ip_address(
        ip_address: Ipv4Addr,
        settings: &Settings,
    ) -> Result<()> {
        let ip_str = ip_address.to_string();

        if fs::write(settings.get_previous_ip_file(), ip_str).is_err() {
            Err(eyre!("Unable to update previous IP address: '{ip_address}'"))
        } else {
            Ok(())
        }
    }

    fn determine_variant(
        new_ip_address: Ipv4Addr,
        previous_ip_address: Option<Ipv4Addr>,
    ) -> Self {
        if let Some(previous_ip_address) = previous_ip_address {
            if new_ip_address == previous_ip_address {
                Self::Unchanged(new_ip_address)
            } else {
                Self::Changed { new_ip_address, previous_ip_address }
            }
        } else {
            Self::New(new_ip_address)
        }
    }

    fn get_previous_ip_address(
        settings: &Settings,
    ) -> Result<Option<Ipv4Addr>> {
        read_file_optional(&settings.get_previous_ip_file())
            .map(|s| Ok(s.parse()?))
            .transpose()
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
