use log::info;

use crate::CliError;

pub(crate) fn process_assets() -> Result<(), CliError> {
    info!("Processing assets: 'json'");
    Ok(())
}