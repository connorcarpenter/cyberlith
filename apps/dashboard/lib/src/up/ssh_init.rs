use std::time::Duration;

use log::{info, warn};
use vultr::VultrError;

use crate::get_static_ip;
use crate::utils::run_command;

pub fn ssh_init() -> Result<(), VultrError> {
    remove_existing_known_host()?;

    loop {
        match add_known_host() {
            Ok(()) => break,
            Err(err) => {
                warn!("error adding known host .. (expect a number of `getaddrinfo >>: Name or service not known` errors while instance is starting up)");
                warn!("error: {:?}", err);
                info!("retrying in 5 seconds..");
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
        }
    }

    Ok(())
}

fn remove_existing_known_host() -> Result<(), VultrError> {
    let command_str = format!("ssh-keygen -f /home/connor/.ssh/known_hosts -R {}", get_static_ip());
    run_command(command_str.as_str())
}

fn add_known_host() -> Result<(), VultrError> {
    let command_str = format!("ssh-keyscan -H {} >> /home/connor/.ssh/known_hosts", get_static_ip());
    run_command(command_str.as_str())
}