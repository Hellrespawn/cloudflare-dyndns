use indexmap::IndexMap;
use serde::Deserialize;

use crate::cloudflare_api::record::{DNSRecord, DNSRecordType};

mod fs;

pub use fs::ApplicationConfigLoader;

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ApplicationConfig {
    public_ip_url: String,

    cloudflare_token: String,

    #[serde(flatten)]
    zones: IndexMap<String, ZoneConfig>,
}

impl ApplicationConfig {
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
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
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

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
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
            || record_name.starts_with(&format!("{self_name}."));

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

    fn get_default_config() -> ApplicationConfig {
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

        ApplicationConfig {
            public_ip_url: "https://example.ip".to_owned(),
            cloudflare_token: "12345AB".to_owned(),
            zones,
        }
    }

    #[test]
    fn test_deserialize_default() -> Result<()> {
        let config: ApplicationConfig = toml::from_str(EXAMPLE)?;

        assert_eq!(config, get_default_config());

        Ok(())
    }
}
