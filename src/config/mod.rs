use color_eyre::Result;

mod args;
mod env;
mod paths;
mod settings;

use paths::CONFIG_PATHS;
pub use settings::Settings;

pub use self::args::Args;

pub fn default_settings(args: Args) -> Result<Settings> {
    let system_env = env::Env::from_file(&CONFIG_PATHS.system_config())?;

    let user_env = env::Env::from_file(&CONFIG_PATHS.user_config())?;

    let settings = match (system_env, user_env) {
        (None, None) => Settings::from_args(args, CONFIG_PATHS.user_config()),
        (None, Some(env)) | (Some(env), None) => {
            Settings::from_args_and_env(args, env)
        },
        (Some(system_env), Some(user_env)) => {
            Settings::from_args_and_env(args, system_env.merge(user_env))
        },
    }?;

    Ok(settings)
}
