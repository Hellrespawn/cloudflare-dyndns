use color_eyre::eyre::eyre;
use color_eyre::Result;

mod args;
mod env;
mod ip;
mod paths;
mod settings;
mod zone;

use paths::CONFIG_PATHS;
pub use settings::Settings;

pub use self::args::Args;

pub fn default_settings(args: Args) -> Result<Settings> {
    let env = if let Some(config_file) = &args.config_file {
        let env = env::Env::from_file(config_file)?
            .ok_or(eyre!("Unable to read config from file: {}", config_file))?;

        Some(env)
    } else {
        let system_env = env::Env::from_file(&CONFIG_PATHS.system_config())?;

        let user_env = env::Env::from_file(&CONFIG_PATHS.user_config())?;

        match (system_env, user_env) {
            (None, None) => None,
            (None, Some(env)) | (Some(env), None) => Some(env),
            (Some(system_env), Some(user_env)) => {
                Some(system_env.merge(user_env))
            },
        }
    };

    let settings = if let Some(env) = env {
        Settings::from_args_and_env(args, env)
    } else {
        Settings::from_args(args, CONFIG_PATHS.user_config())
    }?;

    Ok(settings)
}
