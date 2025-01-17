mod fs;

use std::net::Ipv4Addr;

pub use fs::{IpCacheReader, IpCacheWriter};
use indexmap::IndexMap;

/// Caches the latest IP for a give zone ID.
#[derive(Debug, Default, Clone)]
pub struct IpCache {
    cache: IndexMap<String, Ipv4Addr>,
}

#[derive(Debug, Clone, Copy)]
pub enum IpCacheResult {
    New,
    Unchanged,
    Changed { previous_ip_address: Ipv4Addr },
}

impl IpCache {
    #[must_use]
    pub fn new(cache: IndexMap<String, Ipv4Addr>) -> Self {
        Self { cache }
    }

    pub fn handle_ip(
        &mut self,
        zone_id: &str,
        ip_address: Ipv4Addr,
    ) -> IpCacheResult {
        let cached = self
            .cache
            .get(zone_id)
            .is_some_and(|saved_ip| *saved_ip == ip_address);

        if cached {
            IpCacheResult::Unchanged
        } else {
            let previous_ip_address =
                self.cache.insert(zone_id.to_owned(), ip_address);

            if let Some(previous_ip_address) = previous_ip_address {
                IpCacheResult::Changed { previous_ip_address }
            } else {
                IpCacheResult::New
            }
        }
    }

    #[must_use]
    pub fn iter(&self) -> indexmap::map::Iter<'_, String, Ipv4Addr> {
        self.cache.iter()
    }
}

impl<'a> IntoIterator for &'a IpCache {
    type IntoIter = indexmap::map::Iter<'a, String, Ipv4Addr>;
    type Item = (&'a String, &'a Ipv4Addr);

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
