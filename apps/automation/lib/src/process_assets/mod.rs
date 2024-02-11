// mod json;
// mod convert;

use crate::CliError;

pub fn process_assets(env_val: &str) -> Result<(), CliError> {
    match env_val {
        // "json" => json::process_assets(),
        "dev" => Err(CliError::Message("not implemented".to_string())),
        "stage" => Err(CliError::Message("not implemented".to_string())),
        "prod" => Err(CliError::Message("not implemented".to_string())),
        _ => Err(CliError::Message("invalid environment".to_string())),
    }
}