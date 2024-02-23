use std::net::Ipv4Addr;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use fs_err as fs;

use crate::{read_file_optional, Settings};

pub enum IpAddressChange {
    FirstRun(Ipv4Addr),
    Unchanged(Ipv4Addr),
    Changed { new_ip_address: Ipv4Addr, previous_ip_address: Ipv4Addr },
}

impl IpAddressChange {
    pub fn new(
        new_ip_address: Ipv4Addr,
        settings: &Settings,
    ) -> Result<IpAddressChange> {
        let previous_ip_address = Self::get_previous_ip_address(settings)?;

        Ok(Self::determine_variant(new_ip_address, previous_ip_address))
    }

    pub fn get_new_ip_address(&self, force: bool) -> Option<Ipv4Addr> {
        match self {
            IpAddressChange::Changed { new_ip_address, .. }
            | IpAddressChange::FirstRun(new_ip_address) => {
                Some(*new_ip_address)
            },
            IpAddressChange::Unchanged(previous_ip_address) => {
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

        let result = fs::create_dir_all(settings.get_config_dir())
            .and_then(|()| fs::write(settings.get_previous_ip_file(), ip_str));

        if let Err(err) = result {
            Err(eyre!("Unable to update previous IP address: '{ip_address}'\nError: {err}"))
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
            Self::FirstRun(new_ip_address)
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

impl std::fmt::Display for IpAddressChange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IpAddressChange::FirstRun(new_ip_address) => write!(f, "IP address: '{new_ip_address}'"),
            IpAddressChange::Unchanged(previous_ip_address) => write!(f, "IP address unchanged: '{previous_ip_address}'"),
            IpAddressChange::Changed { new_ip_address, previous_ip_address } => write!(f,
                "IP address updated: '{previous_ip_address}' => '{new_ip_address}'"
            ),
        }
    }
}
