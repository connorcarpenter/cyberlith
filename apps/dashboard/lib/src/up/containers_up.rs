
use std::time::Duration;

use crossbeam_channel::TryRecvError;
use log::{info, warn};
use openssh::Session;
use vultr::VultrError;

use crate::utils::{run_command, run_ssh_command, ssh_session_close, ssh_session_create, thread_init_compat};

pub fn containers_up() -> Result<(), VultrError> {
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

    // upload images to container registry
    images_push().await?;

    // ssh into server
    ssh_into_server_to_pull_and_start_containers().await?;

    return Ok(());
}

async fn ssh_into_server_to_pull_and_start_containers() -> Result<(), VultrError> {

    // ssh in
    let session = ssh_session_create().await?;

    // remove network
    remove_network(&session).await?;

    // stop containers
    containers_stop(&session).await?;

    // pull images
    images_pull(&session).await?;

    // create network
    create_network(&session).await?;

    // start containers
    containers_start(&session).await?;

    // close ssh
    ssh_session_close(session).await?;

    info!("SSH session closed");

    Ok(())
}

async fn images_push() -> Result<(), VultrError> {

    run_command("containers", "docker login https://sjc.vultrcr.com/primary -u 9c02a1b0-c8b0-498a-9b92-28bb6dd14cef -p 7qJZ7EzVFCaMLpax5BL84bj8GZzDDZTb6WzU").await?;

    image_push("content").await?;
    image_push("orchestrator").await?;
    image_push("region").await?;
    image_push("session").await?;
    image_push("world").await?;

    Ok(())
}

async fn images_pull(session: &Session) -> Result<(), VultrError> {

    run_ssh_command(&session, "docker login https://sjc.vultrcr.com/primary -u 9c02a1b0-c8b0-498a-9b92-28bb6dd14cef -p 7qJZ7EzVFCaMLpax5BL84bj8GZzDDZTb6WzU").await?;

    image_pull(session, "content").await?;
    image_pull(session, "orchestrator").await?;
    image_pull(session, "region").await?;
    image_pull(session, "session").await?;
    image_pull(session, "world").await?;

    Ok(())
}

async fn create_network(session: &Session) -> Result<(), VultrError> {

    run_ssh_command(session, "docker network create primary_network").await?;

    Ok(())
}

async fn remove_network(session: &Session) -> Result<(), VultrError> {

    if let Err(err) = run_ssh_command(session, "docker network rm primary_network").await {
        warn!("ignoring error while creating network: {:?}", err);
    }

    Ok(())
}

async fn containers_start(session: &Session) -> Result<(), VultrError> {

    container_start(session, "content", "-p 80:14196/tcp").await?;
    container_start(session, "orchestrator", "-p 14197:14197/tcp").await?;
    container_start(session, "region", "-p 14198:14198/tcp").await?;
    container_start(session, "session", "-p 14200:14200/tcp -p 14201:14201/udp").await?;
    container_start(session, "world", "-p 14203:14203/tcp -p 14204:14204/udp").await?;

    Ok(())
}

async fn containers_stop(session: &Session) -> Result<(), VultrError> {

    container_stop(session, "content").await?;
    container_stop(session, "orchestrator").await?;
    container_stop(session, "region").await?;
    container_stop(session, "session").await?;
    container_stop(session, "world").await?;

    Ok(())
}

async fn image_push(image_name: &str) -> Result<(), VultrError> {
    run_command("containers", format!("docker tag {}_image:latest sjc.vultrcr.com/primary/{}_image:latest", image_name, image_name).as_str()).await?;
    run_command("containers", format!("docker push sjc.vultrcr.com/primary/{}_image:latest", image_name).as_str()).await?;
    run_command("containers", format!("docker rmi {}_image:latest", image_name).as_str()).await?;
    Ok(())
}

async fn image_pull(session: &Session, image_name: &str) -> Result<(), VultrError> {

    run_ssh_command(session, format!("docker pull sjc.vultrcr.com/primary/{}_image:latest", image_name).as_str()).await?;

    Ok(())
}

async fn container_start(session: &Session, app_name: &str, ports: &str) -> Result<(), VultrError> {

    run_ssh_command(session, format!("docker run -d --name {}_server --network primary_network {} sjc.vultrcr.com/primary/{}_image", app_name, ports, app_name).as_str()).await?;

    Ok(())
}

async fn container_stop(session: &Session, app_name: &str) -> Result<(), VultrError> {

    // kill/stop image
    // TODO: should stop instead of kill?
    if let Err(ignored_err) = run_ssh_command(session, format!("docker kill {}_server", app_name).as_str()).await {
        warn!("ignoring error while killing container: {:?}", ignored_err);
    }

    // remove image
    if let Err(ignored_err) = run_ssh_command(session, format!("docker rm {}_server", app_name).as_str()).await {
        warn!("ignoring error while removing container: {:?}", ignored_err);
    }

    Ok(())
}