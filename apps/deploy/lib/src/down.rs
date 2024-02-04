use std::time::Duration;

use log::info;
use vultr::{VultrApi, VultrError};

use crate::{utils::{thread_init_compat, check_channel}, get_api_key};

pub fn down() -> Result<(), VultrError> {
    info!("Stopping instance");

    let rcvr = thread_init_compat(stop_instance);
    let mut rdy = false;

    loop {
        std::thread::sleep(Duration::from_secs(5));

        check_channel(&rcvr, &mut rdy)?;

        if rdy {
            break;
        }
    }

    info!("Done!");
    Ok(())
}

async fn stop_instance() -> Result<(), VultrError> {
    let api_key = get_api_key();

    let api = VultrApi::new(api_key);

    let instances = api.get_instance_list_async().await?;
    if instances.is_empty() {
        return Err(VultrError::Dashboard("No instances running".to_string()));
    }
    if instances.len() > 1 {
        return Err(VultrError::Dashboard("More than one instance running!".to_string()));
    }
    let instance = instances.first().unwrap();

    api.delete_instance_async(instance.id.clone()).await?;

    Ok(())
}