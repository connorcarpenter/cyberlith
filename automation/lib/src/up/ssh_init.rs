use std::time::Duration;

use logging::{info, warn};
use openssh::Session;

use crate::{
    utils::{run_command, ssh_session_create},
    CliError,
};

pub async fn ssh_init() -> Result<Session, CliError> {
    remove_existing_known_host().await?;

    loop {
        match add_known_host().await {
            Ok(()) => break,
            Err(err) => {
                warn!("error adding known host .. (expect a number of `getaddrinfo >>: Name or service not known` errors while instance is starting up)");
                warn!("error: {:?}", err);
                info!("retrying in 5 seconds..");
                executor::smol::Timer::after(Duration::from_secs(5)).await;
                continue;
            }
        }
    }

    // create ssh session
    let session = ssh_session_create().await?;

    Ok(session)
}

async fn remove_existing_known_host() -> Result<(), CliError> {
    let command_str = "ssh-keygen -f /home/connor/.ssh/known_hosts -R cyberlith.com";
    run_command("SSH_INIT", command_str).await
}

async fn add_known_host() -> Result<(), CliError> {
    let command_str = "ssh-keyscan -H cyberlith.com >> /home/connor/.ssh/known_hosts";
    run_command("SSH_INIT", command_str).await
}
