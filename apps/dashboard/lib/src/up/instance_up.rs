use log::{info, warn};
use vultr::VultrError;

use crate::up::{instance_init::instance_init, instance_start::instance_start, instance_wait::instance_wait, ssh_init::ssh_init};

pub async fn instance_up() -> Result<(), VultrError> {

    // start instance
    info!("Starting instance");
    let instance_id = match instance_start() {
        Ok(instance_id) => {
            info!("Instance started! id is {:?}", instance_id);
            instance_id
        },
        Err(e) => {
            warn!("Error starting instance: {:?}", e);
            return Err(e);
        },
    };

    // wait for instance to be ready
    match instance_wait(&instance_id).await {
        Ok(_) => info!("Instance ready!"),
        Err(e) => {
            warn!("Error waiting for instance: {:?}", e);
            return Err(e);
        },
    }

    // init ssh
    match ssh_init().await {
        Ok(_) => info!("SSH initiated"),
        Err(e) => {
            warn!("SSH not initiated.. error: {:?}", e);
            return Err(e);
        },
    }

    // ssh into instance, set up iptables & docker
    match instance_init().await {
        Ok(_) => info!("SSH and initial commands completed successfully"),
        Err(e) => {
            warn!("SSH and initial commands failed: {:?}", e);
            return Err(e);
        },
    }

    Ok(())
}