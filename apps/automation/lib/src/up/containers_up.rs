use std::{thread, time::Duration};

use crossbeam_channel::TryRecvError;
use logging::{info, warn};
use openssh::Session;

use crate::{
    get_container_registry_creds, get_container_registry_url,
    utils::{
        run_command, run_ssh_command, ssh_session_close, ssh_session_create, thread_init_compat,
    },
    CliError,
};

pub fn containers_up() -> Result<(), CliError> {
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
    // upload images to container registry
    images_push().await?;

    // ssh into server
    ssh_into_server_to_pull_and_start_containers().await?;

    return Ok(());
}

async fn ssh_into_server_to_pull_and_start_containers() -> Result<(), CliError> {
    // ssh in
    let session = ssh_session_create().await?;

    // stop containers
    containers_stop(&session).await?;

    // remove network
    remove_network(&session).await?;

    // pull images
    images_pull(&session).await?;

    // create network
    create_network(&session).await?;

    // start containers
    containers_start(&session).await?;

    // prune images
    images_prune(&session).await?;

    // close ssh
    ssh_session_close(session).await?;

    info!("SSH session closed");

    Ok(())
}

async fn images_push() -> Result<(), CliError> {
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

    image_push("content").await?;
    image_push("gateway").await?;
    image_push("region").await?;
    image_push("session").await?;
    image_push("world").await?;

    Ok(())
}

async fn images_pull(session: &Session) -> Result<(), CliError> {
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

    image_pull(session, "content").await?;
    image_pull(session, "gateway").await?;
    image_pull(session, "region").await?;
    image_pull(session, "session").await?;
    image_pull(session, "world").await?;

    Ok(())
}

async fn create_network(session: &Session) -> Result<(), CliError> {
    run_ssh_command(session, "docker network create primary_network").await?;

    Ok(())
}

async fn remove_network(session: &Session) -> Result<(), CliError> {
    if let Err(err) = run_ssh_command(session, "docker network rm primary_network").await {
        warn!("ignoring error while removing network: {:?}", err);
    }

    Ok(())
}

async fn containers_start(session: &Session) -> Result<(), CliError> {
    container_create_and_start(session, "content", "-p 80:80/tcp").await?;
    container_create_and_start(session, "gateway", "-p 14197:14197/tcp").await?;
    container_create_and_start(session, "region", "-p 14198:14198/tcp").await?;
    container_create_and_start(session, "session", "-p 14200:14200/tcp -p 14201:14201/udp").await?;
    container_create_and_start(session, "world", "-p 14203:14203/tcp -p 14204:14204/udp").await?;

    Ok(())
}

async fn images_prune(session: &Session) -> Result<(), CliError> {
    run_ssh_command(session, "echo 'y' | docker image prune -a").await?;

    Ok(())
}

async fn containers_stop(session: &Session) -> Result<(), CliError> {
    container_stop_and_remove(session, "content").await?;
    container_stop_and_remove(session, "gateway").await?;
    container_stop_and_remove(session, "region").await?;
    container_stop_and_remove(session, "session").await?;
    container_stop_and_remove(session, "world").await?;

    Ok(())
}

pub async fn image_push(image_name: &str) -> Result<(), CliError> {
    run_command(
        "containers",
        format!(
            "docker tag {}_image:latest {}/{}_image:latest",
            image_name,
            get_container_registry_url(),
            image_name
        )
        .as_str(),
    )
    .await?;
    run_command(
        "containers",
        format!(
            "docker push {}/{}_image:latest",
            get_container_registry_url(),
            image_name
        )
        .as_str(),
    )
    .await?;
    run_command(
        "containers",
        format!("docker rmi {}_image:latest", image_name).as_str(),
    )
    .await?;
    Ok(())
}

pub async fn image_pull(session: &Session, image_name: &str) -> Result<(), CliError> {
    run_ssh_command(
        session,
        format!(
            "docker pull {}/{}_image:latest",
            get_container_registry_url(),
            image_name
        )
        .as_str(),
    )
    .await?;

    Ok(())
}

pub async fn container_create_and_start(
    session: &Session,
    app_name: &str,
    ports: &str,
) -> Result<(), CliError> {
    run_ssh_command(
        session,
        format!(
            "docker run -d --name {}_server --network primary_network {} {}/{}_image",
            app_name,
            ports,
            get_container_registry_url(),
            app_name
        )
        .as_str(),
    )
    .await?;

    Ok(())
}

pub async fn container_stop_and_remove(session: &Session, app_name: &str) -> Result<(), CliError> {
    // kill/stop container
    // TODO: should stop instead of kill?
    if let Err(ignored_err) =
        run_ssh_command(session, format!("docker kill {}_server", app_name).as_str()).await
    {
        warn!("ignoring error while killing container: {:?}", ignored_err);
    }

    // remove container
    if let Err(ignored_err) =
        run_ssh_command(session, format!("docker rm {}_server", app_name).as_str()).await
    {
        warn!("ignoring error while removing container: {:?}", ignored_err);
    }

    Ok(())
}
