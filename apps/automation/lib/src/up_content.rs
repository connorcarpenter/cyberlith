use std::{thread, time::Duration};

use crossbeam_channel::TryRecvError;
use log::{info, warn};

use crate::{
    containers_up::{
        container_create_and_start, container_stop_and_remove, image_pull, image_push,
    },
    get_container_registry_creds, get_container_registry_url, server_build,
    utils::{
        check_channel, run_command, run_ssh_command, ssh_session_close, ssh_session_create,
        thread_init, thread_init_compat,
    },
    CliError,
};

pub fn up_content() -> Result<(), CliError> {
    let content_rcvr = thread_init(server_build::server_build_content);
    let mut content_rdy = false;

    loop {
        thread::sleep(Duration::from_secs(5));

        check_channel(&content_rcvr, &mut content_rdy)?;

        if content_rdy {
            break;
        }
    }

    containers_up()?;

    info!("Done!");
    Ok(())
}

fn containers_up() -> Result<(), CliError> {
    let rcvr = thread_init_compat(containers_up_impl);

    loop {
        thread::sleep(Duration::from_secs(5));

        match rcvr.try_recv() {
            Ok(result) => return result,
            Err(TryRecvError::Disconnected) => warn!("containers receiver disconnected!"),
            _ => {}
        }
    }
}

async fn containers_up_impl() -> Result<(), CliError> {
    // login on client
    run_command(
        "containers",
        format!(
            "docker login https://{} {}",
            get_container_registry_url(),
            get_container_registry_creds()
        )
        .as_str(),
    )
    .await?;

    // push image
    image_push("content").await?;

    // ssh into server
    let session = ssh_session_create().await?;

    // stop container
    container_stop_and_remove(&session, "content").await?;

    // login on server
    run_ssh_command(
        &session,
        format!(
            "docker login https://{} {}",
            get_container_registry_url(),
            get_container_registry_creds()
        )
        .as_str(),
    )
    .await?;

    // pull new image
    image_pull(&session, "content").await?;

    // start container
    container_create_and_start(&session, "content", "-p 80:80/tcp").await?;

    // close ssh
    ssh_session_close(session).await?;

    info!("SSH session closed");

    return Ok(());
}
