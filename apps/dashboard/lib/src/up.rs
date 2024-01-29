use std::{path::Path, process::Command as LocalCommand, time::Duration};

use log::{info, warn};
use vultr::{VultrApi, VultrError, VultrInstanceType};
use openssh::{KnownHosts, SessionBuilder, Session};
use async_compat::Compat;
use crossbeam_channel::bounded;

use crate::{executor, get_api_key, get_static_ip};

pub fn up() {

    // thread A:

    // start instance
    info!("Starting instance");
    let instance_id = match start_instance() {
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
    match wait_for_instance_ready(&instance_id) {
        Ok(_) => info!("Instance ready!"),
        Err(e) => {
             warn!("Error waiting for instance: {:?}", e);
        },
    }

    // ssh into instance, set up iptables & docker
    match ssh_and_run_initial_commands() {
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

fn start_instance() -> Result<String, VultrError> {

    let api_key = get_api_key();

    let api = VultrApi::new(api_key);

    let instances = api.get_instance_list()?;
    if instances.len() > 0 {
        return Err(VultrError::Dashboard("Instance already running".to_string()));
    }

    // get region id
    let regions = api.get_regions()?;
    let region = regions
        .iter()
        .find(|r| r.city == "Chicago" && r.country == "US" && r.continent == "North America")
        .ok_or(VultrError::Dashboard("No region found".to_string()))?;
    let region_id = region.id.clone();
    info!("found region id: {}", region_id);

    // get plan id
    let plans = api.get_plans()?;
    let plan = plans
        .iter()
        .find(|p| p.monthly_cost == 5.0 && p.vcpu_count == 1 && p.plan_type == "vc2" && p.ram ==1024 && p.disk == 25.0 && p.bandwidth == 1024.0)
        .ok_or(VultrError::Dashboard("No plan found".to_string()))?;
    let plan_id = plan.id.clone();
    info!("found plan id: {:?}", plan_id);

    // // get os id
    let oses = api.get_os_list()?;
    let os = oses
        .iter()
        .find(|o| o.family.contains("ubuntu") && o.arch == "x64" && o.name == "Ubuntu 22.04 LTS x64")
        .ok_or(VultrError::Dashboard("No OS found".to_string()))?;
    let os_id = os.id;
    info!("found OS id: {:?}", os_id);

    // get ssh key id
    let ssh_keys = api.get_sshkey_list()?;
    let ssh_key = ssh_keys
        .iter()
        .find(|k| k.name == "Primary")
        .ok_or(VultrError::Dashboard("No SSH key found".to_string()))?;
    let ssh_key_id = ssh_key.id.clone();
    info!("found ssh key id: {:?}", ssh_key_id);

    // get reserved ip id
    let reserved_ips = api.get_reserved_ip_list()?;
    let reserved_ip = reserved_ips
        .iter()
        .find(|i| i.label == "Primary")
        .ok_or(VultrError::Dashboard("No reserved IP found".to_string()))?;
    let reserved_ip_id = reserved_ip.id.clone();
    info!("found reserved ip id: {:?}", reserved_ip_id);

    // get ubuntu server iso id
    // let isos = api.get_iso_list()?;
    // let iso = isos
    //     .iter()
    //     .find(|i| i.filename == "ubuntu-22.04.3-live-server-amd64.iso")
    //     .ok_or(VultrError::Dashboard("No ISO found".to_string()))?;
    // let iso_id = iso.id.clone();
    // info!("found iso id: {:?}", iso_id);

    // create instance
    let instance = api
        .create_instance(
            &region_id,
            &plan_id,
            VultrInstanceType::OS(os_id),
        )
        .hostname("primaryserver")
        .label("Primary Server")
        .reserved_ipv4(reserved_ip_id)
        .sshkey_id(&ssh_key_id)
        .enable_ipv6(false)
        .backups(false)
        .ddos_protection(false)
        .activation_email(false)

        .run()?;

    Ok(instance.id)
}

fn wait_for_instance_ready(instance_id: &str) -> Result<(), VultrError> {
    let api_key = get_api_key();

    let api = VultrApi::new(api_key);

    loop {

        match api.get_instance(instance_id) {
            Ok(instance) => {
                info!("instance status: {:?}", instance.status);

                if instance.status == "active" {
                    return Ok(());
                }
            }
            Err(err) => {
                warn!("error getting instance: {:?}", err);
                continue;
            }
        }

        std::thread::sleep(Duration::from_secs(5));
    }
}

fn ssh_and_run_initial_commands() -> Result<(), VultrError> {

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

    let (sender, receiver) = bounded(1);

    executor::spawn(Compat::new(async move {
        let result = ssh_and_run_initial_commands_async().await;
        sender.send(result).expect("failed to send result");
    }))
        .detach();

    loop {
        std::thread::sleep(Duration::from_secs(5));
        if let Ok(result) = receiver.try_recv() {
            return result;
        } else {
            // keep looping till thread finishes
            continue;
        }
    }
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

async fn ssh_and_run_initial_commands_async() -> Result<(), VultrError> {

    info!("preparing to SSH into instance");

    let key_path = Path::new("~/Work/cyberlith/.vultr/vultrkey");

    let ssh_path = format!("ssh://root@{}", get_static_ip());

    let session = SessionBuilder::default()
        .known_hosts_check(KnownHosts::Accept)
        .keyfile(key_path)
        .connect(ssh_path)
        .await
        .map_err(|err| VultrError::Dashboard(err.to_string()))?;

    setup_iptables(&session).await?;
    setup_docker(&session).await?;

    session.close()
        .await
        .map_err(|err| VultrError::Dashboard(err.to_string()))?;

    info!("SSH session closed");

    Ok(())
}

async fn setup_iptables(session: &Session) -> Result<(), VultrError> {

    info!("Setting up IPTables");

    info!("# allow established connections");
    run_ssh_command(&session, "sudo iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT").await?;

    info!("# allow ssh");
    run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport ssh -j ACCEPT").await?;

    info!("# allow loopback");
    run_ssh_command(&session, "sudo iptables -I INPUT 1 -i lo -j ACCEPT").await?;

    info!("# allow port 80 (content server)");
    run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT").await?;

    info!("# allow port 14197 (orchestrator)");
    run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport 14197 -j ACCEPT").await?;

    info!("# allow port 14200 (session signal)");
    run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport 14200 -j ACCEPT").await?;

    info!("# allow port 14201 (session webrtc)");
    run_ssh_command(&session, "sudo iptables -A INPUT -p udp --dport 14201 -j ACCEPT").await?;

    info!("# allow port 14203 (world signal)");
    run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport 14203 -j ACCEPT").await?;

    info!("# allow port 14204 (world webrtc)");
    run_ssh_command(&session, "sudo iptables -A INPUT -p udp --dport 14204 -j ACCEPT").await?;

    Ok(())
}

async fn setup_docker(session: &Session) -> Result<(), VultrError> {

    info!("# update");
    run_ssh_command(&session, "sudo apt-get update").await?;

    info!("# install dependencies");
    run_ssh_command(&session, "sudo apt-get install ca-certificates curl -y").await?;

    info!("# install keyring");
    run_ssh_command(&session, "sudo install -m 0755 -d /etc/apt/keyrings").await?;

    info!("# download GPG key and install");
    run_ssh_command(&session, "sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc").await?;

    info!("# set permissions on keyring");
    run_ssh_command(&session, "sudo chmod a+r /etc/apt/keyrings/docker.asc").await?;

    info!("# add docker to apt sources?");
    run_ssh_raw_command(&session, "echo \"deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo \"$VERSION_CODENAME\") stable\" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null").await?;

    info!("# update");
    run_ssh_command(&session, "sudo apt-get update").await?;

    info!("# install docker packages");
    run_ssh_command(&session, "sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin -y").await?;

    info!("# add user to docker group");
    loop {
        match run_ssh_command(&session, "sudo usermod -aG docker root").await {
            Ok(()) => {
                break;
            },
            Err(err) => {
                warn!("error adding user to docker group: {:?}", err);
                info!("retrying after 5 seconds..");
                smol::Timer::after(Duration::from_secs(5)).await;
            }
        }
    }

    info!("# test that docker works without sudo");
    run_ssh_command(&session, "docker version").await?;

    Ok(())
}

async fn run_ssh_command(session: &Session, command_str: &str) -> Result<(), VultrError> {
    info!("-> {}", command_str);

    let commands: Vec<String> = command_str.split(" ").map(|thestr| thestr.to_string()).collect();

    let mut command = session.command(&commands[0]);
    for i in 1..commands.len() {
        command.arg(&commands[i]);
    }

    let output = command.output().await.map_err(|err| VultrError::Dashboard(err.to_string()))?;
    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        info!("<- {}", result);
        return Ok(());
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(VultrError::Dashboard(format!("Command Error: {}", error_message)));
    }
}

async fn run_ssh_raw_command(session: &Session, command_str: &str) -> Result<(), VultrError> {
    info!("-> {}", command_str);

    let mut raw_command = session.raw_command(command_str);
    let output = raw_command.output().await.map_err(|err| VultrError::Dashboard(err.to_string()))?;
    if output.status.success() {
        let result = String::from_utf8_lossy(&output.stdout);
        info!("<- {}", result);
        return Ok(());
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        return Err(VultrError::Dashboard(format!("Command Error: {}", error_message)));
    }
}