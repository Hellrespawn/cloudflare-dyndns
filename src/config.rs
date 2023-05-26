use anyhow::{anyhow, bail, Result};
use fs_err as fs;
use std::collections::HashMap;
use std::path::Path;

const FILENAME: &str = "cloudflare-dyndns.conf";

#[derive(Debug)]
pub struct Config {
    pub token: String,
    pub zone_id: String,
}

impl Config {
    pub fn read() -> Result<Config> {
        let mut map = HashMap::new();

        Self::read_file(
            &format!("/etc/cloudflare-dyndns/{}", FILENAME),
            &mut map,
        )?;

        let config_dir = dirs::config_dir()
            .ok_or(anyhow!("Unable to read user configuration directory."))?;

        Self::read_file(config_dir.join(FILENAME), &mut map)?;

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

    fn read_file<P: AsRef<Path>>(
        path: P,
        map: &mut HashMap<String, String>,
    ) -> Result<()> {
        let file_body = match fs::read_to_string(path) {
            Ok(string) => string,
            Err(error) => match error.kind() {
                std::io::ErrorKind::NotFound => return Ok(()),
                _ => bail!(error),
            },
        };

        for line in file_body.lines() {
            let (k, v) = line
                .trim()
                .split_once('=')
                .ok_or(anyhow!("Line '{}' is not a 'KEY=VALUE' pair.", line))?;

            map.insert(k.trim().to_uppercase(), v.trim().to_owned());
        }

        Ok(())
    }
}
