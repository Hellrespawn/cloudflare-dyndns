use crate::{read_file_optional, SYSTEM_CONFIG_PATH, USER_CONFIG_PATH};
use anyhow::{anyhow, Result};
use std::collections::HashMap;

#[derive(Debug)]
pub struct Config {
    pub token: String,
    pub zone_id: String,
}

impl Config {
    pub fn read() -> Result<Config> {
        let mut map = HashMap::new();

        if let Some(file_body) = read_file_optional(&SYSTEM_CONFIG_PATH) {
            Self::get_map_from_string(&file_body, &mut map)?;
        }

        if let Some(file_body) = read_file_optional(&USER_CONFIG_PATH) {
            Self::get_map_from_string(&file_body, &mut map)?;
        }

        Self::from_hashmap(&map)
    }

    fn get_map_from_string(
        file_body: &str,
        map: &mut HashMap<String, String>,
    ) -> Result<()> {
        for line in file_body.lines() {
            let (k, v) = line
                .trim()
                .split_once('=')
                .ok_or(anyhow!("Line '{}' is not a 'KEY=VALUE' pair.", line))?;

            map.insert(k.trim().to_uppercase(), v.trim().to_owned());
        }

        Ok(())
    }

    fn from_hashmap(map: &HashMap<String, String>) -> Result<Self> {
        let token = map
            .get("CLOUDFLARE_TOKEN")
            .ok_or(anyhow!("CLOUDFLARE_TOKEN is not set."))?
            .clone();

        let id = map
            .get("CLOUDFLARE_ZONE_ID")
            .ok_or(anyhow!("CLOUDFLARE_ZONE_ID is not set."))?
            .clone();

        Ok(Config { token, zone_id: id })
    }
}
