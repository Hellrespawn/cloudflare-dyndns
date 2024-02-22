use camino::Utf8PathBuf;
use color_eyre::eyre::eyre;
use color_eyre::Result;
use once_cell::sync::Lazy;

use crate::PKG_NAME;

pub static CONFIG_PATHS: Lazy<ConfigPaths> = Lazy::new(|| {
    ConfigPaths::new().expect("Unable to init configuration paths.")
});

#[derive(Debug)]
pub struct ConfigPaths {
    system: Utf8PathBuf,
    user: Utf8PathBuf,
}

impl ConfigPaths {
    fn new() -> Result<Self> {
        let system_config_directory =
            Utf8PathBuf::from(format!("/etc/{PKG_NAME}"));

        let user_config_directory: Utf8PathBuf = dirs::config_dir()
            .ok_or(eyre!("Unable to read user config directory."))?
            .join(PKG_NAME)
            .try_into()?;

        Ok(ConfigPaths {
            system: system_config_directory,
            user: user_config_directory,
        })
    }

    pub fn system_config(&self) -> Utf8PathBuf {
        self.system.join(format!("{PKG_NAME}.conf"))
    }

    pub fn user_config(&self) -> Utf8PathBuf {
        self.user.join(format!("{PKG_NAME}.conf"))
    }
}

impl std::fmt::Display for ConfigPaths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "  {}\n  {}", CONFIG_PATHS.system, CONFIG_PATHS.user)
    }
}
