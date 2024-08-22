use camino::Utf8PathBuf;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use indexmap::IndexMap;
use nix::unistd::geteuid;
use serde::Deserialize;

use crate::PKG_NAME;

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct Config {
    public_ip_url: String,

    cloudflare_token: String,

    #[serde(flatten)]
    zones: IndexMap<String, ZoneConfig>,
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

    pub fn load_config() -> Result<Config> {
        let default_config_file = Self::default_config_file()?;

        let config_paths =
            [default_config_file, Utf8PathBuf::from("config.toml")];

        for config_path in &config_paths {
            if config_path.is_file() {
                let contents = fs_err::read_to_string(config_path)?;
                let config = toml::from_str(&contents)?;

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
            config_dir = Utf8PathBuf::from(format!("/etc/{PKG_NAME}"));
        }

        Ok(config_dir)
    }

    pub fn default_config_file() -> Result<Utf8PathBuf> {
        Ok(Self::default_config_dir()?.join(format!("{PKG_NAME}.conf")))
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq)]
pub struct ZoneConfig {
    records: Vec<RecordConfig>,
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

#[derive(Deserialize, Debug, PartialEq, Eq, Default)]
pub enum DNSRecordType {
    #[default]
    A,
    AAAA,
    MX,
    TXT,
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
        }
    }

    #[test]
    fn test_deserialize_default() -> Result<()> {
        let config: Config = toml::from_str(EXAMPLE)?;

        assert_eq!(config, get_default_config());

        Ok(())
    }
}
