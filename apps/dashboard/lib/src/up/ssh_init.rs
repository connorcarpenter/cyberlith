use std::{process::Command as LocalCommand, time::Duration};

use log::{info, warn};
use vultr::VultrError;

use crate::get_static_ip;

pub fn ssh_init() -> Result<(), VultrError> {
    remove_existing_known_host()?;

    loop {
        match add_known_host() {
            Ok(()) => break,
            Err(err) => {
                warn!("error adding known host: {:?}", err);
                info!("retrying in 5 seconds..");
                std::thread::sleep(Duration::from_secs(5));
                continue;
            }
        }
    }

    Ok(())
}

fn remove_existing_known_host() -> Result<(), VultrError> {
    let static_ip = get_static_ip();
    info!("(local) -> ssh-keygen -f /home/connor/.ssh/known_hosts -R {}", get_static_ip());
    let output = LocalCommand::new("ssh-keygen")
        .arg("-f")
        .arg("/home/connor/.ssh/known_hosts")
        .arg("-R")
        .arg(static_ip)
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        info!("(local) <- {}", result);
        return Ok(());
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(VultrError::Dashboard(format!("LocalCommand Error: {}", error_message)));
    }
}

fn add_known_host() -> Result<(), VultrError> {
    info!("(local) -> ssh-keyscan -H {} >> /home/connor/.ssh/known_hosts", get_static_ip());
    let output = LocalCommand::new("ssh-keyscan")
        .arg("-H")
        .arg(get_static_ip())
        .arg(">>")
        .arg("/home/connor/.ssh/known_hosts")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        info!("(local) <- {}", result);
        return Ok(());
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(VultrError::Dashboard(format!("LocalCommand Error: {}", error_message)));
    }
}