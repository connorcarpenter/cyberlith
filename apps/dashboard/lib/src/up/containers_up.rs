use std::time::Duration;
use crossbeam_channel::TryRecvError;
use log::{info, warn};
use vultr::VultrError;

use crate::utils::{run_command, thread_init};

pub fn containers_up() -> Result<(), VultrError> {
    let rcvr = thread_init(containers_up_impl);

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

    // upload images to container registry
    images_upload().await?;

    // ssh into server
    ssh_into_server_to_pull_and_start_containers().await?;

    return Ok(());
}

async fn images_upload() -> Result<(), VultrError> {

    run_command("containers", "docker login https://sjc.vultrcr.com/primary -u 9c02a1b0-c8b0-498a-9b92-28bb6dd14cef -p 7qJZ7EzVFCaMLpax5BL84bj8GZzDDZTb6WzU").await?;

    image_upload("content_image").await?;
    image_upload("orchestrator_image").await?;
    image_upload("region_image").await?;
    image_upload("session_image").await?;
    image_upload("world_image").await?;

    Ok(())
}

async fn ssh_into_server_to_pull_and_start_containers() -> Result<(), VultrError> {

    // ssh in

    // stop containers

    // pull images

    // start containers

    Ok(())
}

async fn image_upload(image_name: &str) -> Result<(), VultrError> {
    run_command("containers", format!("docker tag {}:latest sjc.vultrcr.com/primary/{}:latest", image_name, image_name).as_str()).await?;
    run_command("containers", format!("docker push sjc.vultrcr.com/primary/{}:latest", image_name).as_str()).await?;
    run_command("containers", format!("docker rmi {}:latest", image_name).as_str()).await?;
    Ok(())
}