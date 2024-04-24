use std::env;

use bevy_app::{App, Plugin};
use bevy_ecs::system::Resource;
use config_rs::{Config as ConfigRs, ConfigError, File};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
#[allow(unused)]
pub struct GeneralConfig {
    pub env_name: String,
}

#[derive(Debug, Deserialize, Resource, Default)]
#[allow(unused)]
pub struct AppConfig {
    pub general: GeneralConfig,
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
    pub fn new() -> Result<Self, ConfigError> {
        let config_env = env::var("CONFIG_ENV").expect("CONFIG_ENV must be set");

        let config_folder = "src/config";

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
}
