mod instance_start;
mod instance_wait;
mod instance_init;
mod ssh_init;

use log::{info, warn};

use crate::{
    up::{
        instance_start::instance_start,
        instance_init::instance_init,
        ssh_init::ssh_init,
        instance_wait::instance_wait
    }
};

pub fn up() {

    // thread A:

    // start instance
    info!("Starting instance");
    let instance_id = match instance_start() {
        Ok(instance_id) => {
            info!("Instance started! id is {:?}", instance_id);
            instance_id
        },
        Err(e) => {
            warn!("Error starting instance: {:?}", e);
            return;
        },
    };

    // wait for instance to be ready
    match instance_wait(&instance_id) {
        Ok(_) => info!("Instance ready!"),
        Err(e) => {
            warn!("Error waiting for instance: {:?}", e);
        },
    }

    // init ssh
    match ssh_init() {
        Ok(_) => info!("SSH initiated"),
        Err(e) => {
            warn!("SSH not initiated.. error: {:?}", e);
            return;
        },
    }

    // ssh into instance, set up iptables & docker
    match instance_init() {
        Ok(_) => info!("SSH and initial commands completed successfully"),
        Err(e) => {
            warn!("SSH and initial commands failed: {:?}", e);
            return;
        },
    }

    // thread B:

    // build all apps in release mode (multithread this??)

    // turn binaries into dockerimages

    // wait for thread A & thread B to finish..

    // scp dockerimages to instance

    // ssh into instance, start docker containers with new images

    // test?

    info!("Done!");
}




