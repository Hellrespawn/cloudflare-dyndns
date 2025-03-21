use camino::{Utf8Path, Utf8PathBuf};
use color_eyre::Result;
use color_eyre::eyre::eyre;
use nix::unistd::geteuid;
use tracing::{debug, trace};

use crate::PKG_NAME;
use crate::config::ApplicationConfig;

pub struct ApplicationConfigLoader;

impl ApplicationConfigLoader {
    pub fn load_config_from(
        config_path: &Utf8Path,
    ) -> Result<ApplicationConfig> {
        let contents = fs_err::read_to_string(config_path)?;
        let config: ApplicationConfig = toml::from_str(&contents)?;

        debug!("Loaded configuration from {config_path}");

        // TODO? Explicitly handle error on missing file

        Ok(config)
    }

    pub fn default_config_file() -> Result<Utf8PathBuf> {
        Ok(Self::default_config_dir()?.join(Self::default_config_file_name()))
    }

    pub fn default_config_dir() -> Result<Utf8PathBuf> {
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

    #[must_use]
    pub fn default_config_file_name() -> String {
        format!("{PKG_NAME}.toml")
    }
}
