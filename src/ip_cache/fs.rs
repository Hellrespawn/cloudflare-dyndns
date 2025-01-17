use std::fmt::Write;

use camino::Utf8Path;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use indexmap::IndexMap;

static DELIMITER: &str = ";";

use super::IpCache;

pub struct IpCacheReader;

impl IpCacheReader {
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

            Ok(IpCache::new(cache))
        } else if path.exists() {
            Err(eyre!("Cache file path exists, but is not a file!"))
        } else {
            Ok(IpCache::default())
        }
    }
}

pub struct IpCacheWriter;

impl IpCacheWriter {
    pub fn save(&self, ip_cache: &IpCache, path: &Utf8Path) -> Result<()> {
        let body = ip_cache.into_iter().fold(
            String::new(),
            |mut acc, (key, value)| {
                writeln!(acc, "{key}{DELIMITER}{value}").unwrap();
                acc
            },
        );

        fs_err::write(path, body)?;

        Ok(())
    }
}
