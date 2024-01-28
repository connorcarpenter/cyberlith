use log::info;

use vultr::{VultrApi, VultrError};

pub fn down() {
    info!("Stopping instance");
    let result = stop_instance();
    match result {
        Ok(_) => info!("Instance stopped!"),
        Err(e) => info!("Error stopping instance: {:?}", e),
    }
}

fn stop_instance() -> Result<(), VultrError> {
    let api_key = "SNU5BHCV5Y56LAIYUKQPEMEEPZIVIO7APODA";

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