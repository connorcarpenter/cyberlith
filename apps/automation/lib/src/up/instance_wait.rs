use std::time::Duration;

use log::{info, warn};
use vultr::VultrApi;

use crate::{get_api_key, CliError};

pub async fn instance_wait(instance_id: &str) -> Result<(), CliError> {
    let api_key = get_api_key();

    let api = VultrApi::new(api_key);

    loop {
        match api.get_instance_async(instance_id).await {
            Ok(instance) => {
                info!("instance status: {:?}", instance.status);

                if instance.status == "active" {
                    return Ok(());
                }
            }
            Err(err) => {
                warn!("error getting instance: {:?}", err);
                continue;
            }
        }

        smol::Timer::after(Duration::from_secs(5)).await;
    }
}
