

use std::time::Duration;

use crossbeam_channel::{Receiver, TryRecvError};
use log::{info, warn};
use vultr::VultrError;
use crate::containers_up::{container_create_and_start, container_stop_and_remove, image_pull, image_push};
use crate::server_build;
use crate::utils::{check_channel, run_command, run_ssh_command, ssh_session_close, ssh_session_create, thread_init, thread_init_compat};

pub fn up_content() -> Result<(), VultrError> {

    let content_rcvr = thread_init(server_build::server_build_content);
    let mut content_rdy = false;

    loop {
        std::thread::sleep(Duration::from_secs(5));

        check_channel(&content_rcvr, &mut content_rdy)?;

        if content_rdy {
            break;
        }
    }

    containers_up()?;

    info!("Done!");
    Ok(())
}

fn containers_up() -> Result<(), VultrError> {
    let rcvr = thread_init_compat(containers_up_impl);

    loop {
        std::thread::sleep(Duration::from_secs(5));

        match rcvr.try_recv() {
            Ok(result) => return result,
            Err(TryRecvError::Disconnected) => warn!("containers receiver disconnected!"),
            _ => {},
        }
    }
}

async fn containers_up_impl() -> Result<(), VultrError> {

    // login on client (TODO: abstract away this!)
    run_command("containers", "docker login https://sjc.vultrcr.com/primary -u 9c02a1b0-c8b0-498a-9b92-28bb6dd14cef -p 7qJZ7EzVFCaMLpax5BL84bj8GZzDDZTb6WzU").await?;

    // push image
    image_push("content").await?;

    // ssh into server
    let session = ssh_session_create().await?;

    // stop container
    container_stop_and_remove(&session, "content").await?;

    // login on server (TODO: abstract away this!)
    run_ssh_command(&session, "docker login https://sjc.vultrcr.com/primary -u 9c02a1b0-c8b0-498a-9b92-28bb6dd14cef -p 7qJZ7EzVFCaMLpax5BL84bj8GZzDDZTb6WzU").await?;

    // pull new image
    image_pull(&session, "content").await?;

    // start container
    container_create_and_start(&session, "content", "-p 80:80/tcp").await?;

    // close ssh
    ssh_session_close(session).await?;

    info!("SSH session closed");

    return Ok(());
}
