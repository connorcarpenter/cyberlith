use std::time::Duration;

use log::info;
use vultr::VultrApi;

use crate::{utils::{thread_init_compat, check_channel}, get_api_key, CliError};

pub fn down() -> Result<(), CliError> {
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

async fn stop_instance() -> Result<(), CliError> {
    let api_key = get_api_key();

    let api = VultrApi::new(api_key);

    let instances = api.get_instance_list_async().await?;
    if instances.is_empty() {
        return Err(CliError::Message("No instances running".to_string()));
    }
    if instances.len() > 1 {
        return Err(CliError::Message("More than one instance running!".to_string()));
    }
    let instance = instances.first().unwrap();

    api.delete_instance_async(instance.id.clone()).await?;

    Ok(())
}