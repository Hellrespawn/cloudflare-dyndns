use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::eyre::eyre;
use color_eyre::Result;
use indexmap::IndexMap;
use nix::unistd::geteuid;
use serde::Deserialize;
use tracing::{debug, trace};

use crate::cloudflare_api::record::{DNSRecord, DNSRecordType};
use crate::PKG_NAME;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Config {
    public_ip_url: String,

    cloudflare_token: String,

    #[serde(flatten)]
    zones: IndexMap<String, ZoneConfig>,

    #[serde(skip)]
    cache_file: Utf8PathBuf,
}

impl Config {
    #[must_use]
    pub fn public_ip_url(&self) -> &str {
        &self.public_ip_url
    }

    #[must_use]
    pub fn cloudflare_token(&self) -> &str {
        &self.cloudflare_token
    }

    #[must_use]
    pub fn zones(&self) -> &IndexMap<String, ZoneConfig> {
        &self.zones
    }

    #[must_use]
    pub fn cache_file(&self) -> &Utf8Path {
        self.cache_file.as_ref()
    }

    pub fn load_config() -> Result<Config> {
        let default_config_file = Self::default_config_file()?;

        let config_paths =
            [default_config_file, Utf8PathBuf::from("config.toml")];

        for config_path in &config_paths {
            if config_path.is_file() {
                let contents = fs_err::read_to_string(config_path)?;
                let mut config: Config = toml::from_str(&contents)?;

                config.cache_file = config_path.with_extension("cache");

                debug!("Loaded configuration from {config_path}");

                return Ok(config);
            }
        }

        Err(eyre!("Unable to read configuration file at {}", config_paths[0]))
    }

    fn default_config_dir() -> Result<Utf8PathBuf> {
        let mut config_dir: Utf8PathBuf = dirs::config_dir()
            .ok_or(eyre!(
                "Unable to determine config directory for current user."
            ))?
            .try_into()?;

        #[cfg(unix)]
        if geteuid().is_root() {
            trace!("Running as root on unix, using /etc/ instead of $HOME.");
            config_dir = Utf8PathBuf::from(format!("/etc/{PKG_NAME}"));
        }

        Ok(config_dir)
    }

    fn default_config_file() -> Result<Utf8PathBuf> {
        Ok(Self::default_config_dir()?.join(format!("{PKG_NAME}.conf")))
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct ZoneConfig {
    records: Vec<RecordConfig>,
}

impl ZoneConfig {
    #[must_use]
    pub fn records(&self) -> &[RecordConfig] {
        &self.records
    }

    #[must_use]
    pub fn is_record_selected(&self, record: &DNSRecord) -> bool {
        self.records.iter().any(|r| r.match_record(record))
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
#[serde[untagged]]
pub enum RecordConfig {
    Full {
        #[serde(rename = "type", default)]
        record_type: DNSRecordType,
        name: String,
    },
    Name(String),
}

impl RecordConfig {
    fn match_record(&self, record: &DNSRecord) -> bool {
        let record_name = &record.name;
        let self_name = self.name();

        let name_matches = self_name == record_name
            || record_name.starts_with(&format!("{}.", self_name));

        let type_matches = self.record_type() == record.record_type;

        type_matches && name_matches
    }

    #[must_use]
    pub fn name(&self) -> &str {
        match self {
            RecordConfig::Full { name, .. } | RecordConfig::Name(name) => name,
        }
    }

    #[must_use]
    pub fn record_type(&self) -> DNSRecordType {
        match self {
            RecordConfig::Full { record_type, .. } => *record_type,
            RecordConfig::Name(_) => DNSRecordType::A,
        }
    }
}

#[cfg(test)]
mod test {
    use color_eyre::Result;

    use super::*;

    const EXAMPLE: &str = include_str!("../../test/example.toml");

    fn get_default_config() -> Config {
        let mut zones = IndexMap::new();

        zones.insert("example.nl".to_owned(), ZoneConfig {
            records: vec![
                RecordConfig::Full {
                    record_type: DNSRecordType::A,
                    name: "www".to_owned(),
                },
                RecordConfig::Full {
                    record_type: DNSRecordType::A,
                    name: "mail".to_owned(),
                },
                RecordConfig::Name("test".to_owned()),
            ],
        });

        zones.insert("otherexample.com".to_owned(), ZoneConfig {
            records: vec![
                RecordConfig::Full {
                    record_type: DNSRecordType::A,
                    name: "www".to_owned(),
                },
                RecordConfig::Full {
                    record_type: DNSRecordType::A,
                    name: "mail".to_owned(),
                },
                RecordConfig::Name("test".to_owned()),
            ],
        });

        Config {
            public_ip_url: "https://example.ip".to_owned(),
            cloudflare_token: "12345AB".to_owned(),
            zones,
            cache_file: Utf8PathBuf::new(),
        }
    }

    #[test]
    fn test_deserialize_default() -> Result<()> {
        let config: Config = toml::from_str(EXAMPLE)?;

        assert_eq!(config, get_default_config());

        Ok(())
    }
}
