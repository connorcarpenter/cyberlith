use std::env;

use bevy_app::{App, Plugin};
use bevy_ecs::system::Resource;
use config_rs::{Config as ConfigRs, ConfigError, File};
use serde::Deserialize;

#[cfg(target_arch = "wasm32")]
use config_rs::FileFormat;

#[derive(Debug, Deserialize, Default)]
#[allow(unused)]
pub struct GeneralConfig {
    pub env_name: String,
}

#[derive(Debug, Deserialize, Default)]
#[allow(unused)]
pub struct LoginConfig {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Resource, Default)]
#[allow(unused)]
pub struct AppConfig {
    pub general: GeneralConfig,
    pub login: Option<LoginConfig>,
}

#[derive(Resource)]
pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        let config = AppConfig::new().unwrap();

        app.insert_resource(config);
    }
}

impl AppConfig {
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new() -> Result<Self, ConfigError> {
        let config_env = env::var("CONFIG_ENV").expect("CONFIG_ENV must be set");

        let config_folder = "src/app/config";

        let c = ConfigRs::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name(&format!(
                "{}/config_default",
                config_folder
            )))
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .add_source(
                File::with_name(&format!("{}/{}", config_folder, config_env)).required(false),
            )
            .build()?;

        c.try_deserialize()
    }

    #[cfg(target_arch = "wasm32")]
    fn new() -> Result<Self, ConfigError> {
        let c = ConfigRs::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::from_str(
                include_str!("config_default.yaml"),
                FileFormat::Yaml,
            ))
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .add_source(
                File::from_str(
                    include_str!(concat!(std::env!("CONFIG_ENV"), ".yaml")),
                    FileFormat::Yaml,
                )
                .required(false),
            )
            .build()?;

        c.try_deserialize()
    }
}
