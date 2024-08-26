use std::fmt::Write;
use std::net::Ipv4Addr;

use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use indexmap::IndexMap;

static DELIMITER: char = ';';

#[derive(Debug, Default)]
pub struct IpCache {
    cache: IndexMap<String, Ipv4Addr>,
    path: Utf8PathBuf,
}

#[derive(Debug, Clone, Copy)]
pub enum IpCacheResult {
    New,
    Unchanged,
    Changed { previous_ip_address: Ipv4Addr },
}

impl IpCache {
    pub fn save(&self) -> Result<()> {
        self.save_to(&self.path)
    }

    pub fn save_to(&self, path: &Utf8Path) -> Result<()> {
        let body =
            self.cache.iter().fold(String::new(), |mut acc, (key, value)| {
                writeln!(acc, "{key}{DELIMITER}{value}").unwrap();
                acc
            });

        fs_err::write(path, &body)?;

        Ok(())
    }

    pub fn load(path: &Utf8Path) -> Result<IpCache> {
        if path.is_file() {
            let body = fs_err::read_to_string(path)?;

            let mut cache = IndexMap::new();

            for line in body.lines() {
                let (key, value) = line.split_once(DELIMITER).ok_or(eyre!(
                    "Line should contain two values, separated by {DELIMITER}"
                ))?;

                cache.insert(key.to_owned(), value.parse()?);
            }

            Ok(IpCache { cache, path: path.to_owned() })
        } else if path.exists() {
            Err(eyre!("Cache file path exists, but is not a file!"))
        } else {
            Ok(IpCache { cache: IndexMap::new(), path: path.to_owned() })
        }
    }

    pub fn handle_ip(
        &mut self,
        zone_id: &str,
        ip_address: Ipv4Addr,
    ) -> IpCacheResult {
        let changed = !self
            .cache
            .get(zone_id)
            .is_some_and(|saved_ip| *saved_ip == ip_address);

        if changed {
            let previous_ip_address =
                self.cache.insert(zone_id.to_owned(), ip_address);

            if let Some(previous_ip_address) = previous_ip_address {
                IpCacheResult::Changed { previous_ip_address }
            } else {
                IpCacheResult::New
            }
        } else {
            IpCacheResult::Unchanged
        }
    }
}
