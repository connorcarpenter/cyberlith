use log::info;

use vultr::{VultrApi, VultrError};

use crate::get_api_key;

pub fn down() -> Result<(), VultrError> {
    info!("Stopping instance");
    stop_instance()
}

fn stop_instance() -> Result<(), VultrError> {
    let api_key = get_api_key();

    let api = VultrApi::new(api_key);

    let instances = api.get_instance_list()?;
    if instances.is_empty() {
        return Err(VultrError::Dashboard("No instances running".to_string()));
    }
    if instances.len() > 1 {
        return Err(VultrError::Dashboard("More than one instance running!".to_string()));
    }
    let instance = instances.first().unwrap();

    api.delete_instance(instance.id.clone())?;

    Ok(())
}