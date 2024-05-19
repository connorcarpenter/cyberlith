use std::{collections::HashSet, thread, time::Duration};

use executor::smol::channel::TryRecvError;
use logging::{info, warn};
use openssh::Session;

use crate::{
    get_container_registry_creds, get_container_registry_url,
    utils::{
        run_command, run_ssh_command, ssh_session_close, ssh_session_create,
        thread_init_compat_1arg,
    },
    CliError,
};

pub fn containers_up(config: HashSet<String>, image_tag: String) -> Result<(), CliError> {
    let rcvr = thread_init_compat_1arg((config, image_tag), containers_up_impl);

    loop {
        thread::sleep(Duration::from_secs(5));

        match rcvr.try_recv() {
            Ok(result) => return result,
            Err(TryRecvError::Closed) => warn!("containers receiver closed!"),
            _ => {}
        }
    }
}

async fn containers_up_impl(args: (HashSet<String>, String)) -> Result<(), CliError> {
    let config = &args.0;
    let image_tag = &args.1;

    // upload images to container registry
    images_push(config, image_tag).await?;

    // ssh into server
    ssh_into_server_to_pull_and_start_containers(config, image_tag).await?;

    return Ok(());
}

async fn ssh_into_server_to_pull_and_start_containers(
    config: &HashSet<String>,
    image_tag: &str,
) -> Result<(), CliError> {
    // ssh in
    let session = ssh_session_create().await?;

    // stop containers
    containers_stop(config, &session).await?;

    if config.contains("network") {
        // remove network
        remove_network(&session).await?;
    }

    // pull images
    images_pull(config, image_tag, &session).await?;

    if config.contains("network") {
        // create network
        create_network(&session).await?;
    }

    // start containers
    containers_start(config, image_tag, &session).await?;

    // prune images
    images_prune(&session).await?;

    // close ssh
    ssh_session_close(session).await?;

    info!("SSH session closed");

    Ok(())
}

async fn images_push(config: &HashSet<String>, image_tag: &str) -> Result<(), CliError> {
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

    image_push(config, "redirector", image_tag).await?;
    image_push(config, "gateway", image_tag).await?;
    image_push(config, "content", image_tag).await?;
    image_push(config, "auth", image_tag).await?;
    image_push(config, "region", image_tag).await?;
    image_push(config, "session", image_tag).await?;
    image_push(config, "world", image_tag).await?;
    image_push(config, "asset", image_tag).await?;
    image_push(config, "social", image_tag).await?;

    info!("images pushed to container registry");

    Ok(())
}

async fn images_pull(
    config: &HashSet<String>,
    image_tag: &str,
    session: &Session,
) -> Result<(), CliError> {
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

    image_pull(config, session, "redirector", image_tag).await?;
    image_pull(config, session, "gateway", image_tag).await?;
    image_pull(config, session, "content", image_tag).await?;
    image_pull(config, session, "auth", image_tag).await?;
    image_pull(config, session, "region", image_tag).await?;
    image_pull(config, session, "session", image_tag).await?;
    image_pull(config, session, "world", image_tag).await?;
    image_pull(config, session, "asset", image_tag).await?;
    image_pull(config, session, "social", image_tag).await?;

    Ok(())
}

async fn create_network(session: &Session) -> Result<(), CliError> {
    run_ssh_command(session, "docker network create primary_network").await?;

    Ok(())
}

async fn remove_network(session: &Session) -> Result<(), CliError> {
    if let Err(err) = run_ssh_command(session, "docker network rm primary_network").await {
        match &err {
            CliError::Message(inner_msg) => {
                if inner_msg.contains("network primary_network not found") {
                    info!("network `primary_network` does not exist on this instance. Ignore this if the instance was not running before this deployment.");
                } else {
                    warn!("ignoring error while removing network: {:?}", err);
                }
            }
            CliError::Vultr(_) => {
                warn!("ignoring error while removing network: {:?}", err);
            }
        }
    }

    Ok(())
}

async fn containers_start(
    config: &HashSet<String>,
    image_tag: &str,
    session: &Session,
) -> Result<(), CliError> {
    container_create_and_start(config, session, "redirector", image_tag, "-p 80:80/tcp").await?;
    container_create_and_start(config, session, "gateway", image_tag, "-p 443:443/tcp").await?;
    container_create_and_start(config, session, "content", image_tag, "-p 14197:14197/tcp").await?;
    container_create_and_start(config, session, "auth", image_tag, "-p 14206:14206/tcp").await?;
    container_create_and_start(config, session, "region", image_tag, "-p 14198:14198/tcp").await?;
    container_create_and_start(
        config,
        session,
        "session",
        image_tag,
        "-p 14200:14200/tcp -p 14201:14201/udp",
    )
    .await?;
    container_create_and_start(
        config,
        session,
        "world",
        image_tag,
        "-p 14203:14203/tcp -p 14204:14204/udp",
    )
    .await?;
    container_create_and_start(config, session, "asset", image_tag, "-p 14205:14205/tcp").await?;
    container_create_and_start(config, session, "social", image_tag, "-p 14207:14207/tcp").await?;

    Ok(())
}

async fn images_prune(session: &Session) -> Result<(), CliError> {
    run_ssh_command(session, "echo 'y' | docker image prune -a").await?;

    Ok(())
}

async fn containers_stop(config: &HashSet<String>, session: &Session) -> Result<(), CliError> {
    container_stop_and_remove(config, session, "redirector").await?;
    container_stop_and_remove(config, session, "gateway").await?;
    container_stop_and_remove(config, session, "content").await?;
    container_stop_and_remove(config, session, "auth").await?;
    container_stop_and_remove(config, session, "region").await?;
    container_stop_and_remove(config, session, "session").await?;
    container_stop_and_remove(config, session, "world").await?;
    container_stop_and_remove(config, session, "asset").await?;
    container_stop_and_remove(config, session, "social").await?;

    Ok(())
}

pub async fn image_push(
    config: &HashSet<String>,
    image_name: &str,
    image_tag: &str,
) -> Result<(), CliError> {
    if !config.contains(image_name) {
        return Ok(()); // skip this image
    }
    run_command(
        "containers",
        format!(
            "docker tag {}_image:{} {}/{}_image:{}",
            image_name,
            image_tag,
            get_container_registry_url(),
            image_name,
            image_tag,
        )
        .as_str(),
    )
    .await?;
    run_command(
        "containers",
        format!(
            "docker push {}/{}_image:{}",
            get_container_registry_url(),
            image_name,
            image_tag,
        )
        .as_str(),
    )
    .await?;
    // run_command(
    //     "containers",
    //     format!("docker rmi {}_image:latest", image_name).as_str(),
    // )
    // .await?;
    Ok(())
}

pub async fn image_pull(
    config: &HashSet<String>,
    session: &Session,
    image_name: &str,
    image_tag: &str,
) -> Result<(), CliError> {
    if !config.contains(image_name) {
        return Ok(()); // skip this image
    }

    run_ssh_command(
        session,
        format!(
            "docker pull {}/{}_image:{}",
            get_container_registry_url(),
            image_name,
            image_tag,
        )
        .as_str(),
    )
    .await?;

    Ok(())
}

pub async fn container_create_and_start(
    config: &HashSet<String>,
    session: &Session,
    app_name: &str,
    image_tag: &str,
    ports: &str,
) -> Result<(), CliError> {
    if !config.contains(app_name) {
        return Ok(()); // skip this container
    }

    run_ssh_command(
        session,
        format!(
            "docker run -d --name {}_server --network primary_network {} {}/{}_image:{}",
            app_name,
            ports,
            get_container_registry_url(),
            app_name,
            image_tag,
        )
        .as_str(),
    )
    .await?;

    Ok(())
}

pub async fn container_stop_and_remove(
    config: &HashSet<String>,
    session: &Session,
    app_name: &str,
) -> Result<(), CliError> {
    if !config.contains(app_name) {
        return Ok(()); // skip this container
    }

    // kill/stop container
    // TODO: should stop instead of kill?
    if let Err(ignored_err) =
        run_ssh_command(session, format!("docker kill {}_server", app_name).as_str()).await
    {
        match &ignored_err {
            CliError::Message(inner_msg) => {
                if inner_msg.contains("No such container") {
                    info!("container `{}_server` does not exist on this instance. Ignore this if the instance was not running before this deployment.", app_name);
                } else {
                    warn!("error while killing container: {:?}", ignored_err);
                }
            }
            CliError::Vultr(_) => {
                warn!("error while killing container: {:?}", ignored_err);
            }
        }
    }

    // remove container
    if let Err(ignored_err) =
        run_ssh_command(session, format!("docker rm {}_server", app_name).as_str()).await
    {
        match &ignored_err {
            CliError::Message(inner_msg) => {
                if inner_msg.contains("No such container") {
                    info!("container `{}_server` does not exist on this instance. Ignore this if the instance was NOT running before this deployment.", app_name);
                } else {
                    warn!("error while removing container: {:?}", ignored_err);
                }
            }
            CliError::Vultr(_) => {
                warn!("error while removing container: {:?}", ignored_err);
            }
        }
    }

    Ok(())
}
