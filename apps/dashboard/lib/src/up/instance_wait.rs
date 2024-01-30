use std::time::Duration;

use log::{info, warn};
use vultr::{VultrApi, VultrError};

use crate::get_api_key;

pub fn instance_wait(instance_id: &str) -> Result<(), VultrError> {
    let api_key = get_api_key();

    let api = VultrApi::new(api_key);

    loop {

        match api.get_instance(instance_id) {
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

        std::thread::sleep(Duration::from_secs(5));
    }
}