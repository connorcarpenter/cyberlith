use std::path::Path;

use log::info;
use vultr::{VultrApi, VultrError, VultrInstanceType};
use openssh::{KnownHosts, SessionBuilder, Error as OpenSshError, Session};
use async_compat::Compat;

use crate::{executor, get_api_key, get_static_ip};

pub fn up() {

    // thread A:

    // start instance
    info!("Starting instance");
    let result = start_instance();
    match result {
        Ok(instance_id) => info!("Instance started! id is {:?}", instance_id),
        Err(e) => info!("Error starting instance: {:?}", e),
    }

    // wait for instance to be ready
    todo!();

    // ssh into instance, set up iptables & docker
    ssh_and_run_initial_commands();

    // thread B:

    // build all apps in release mode (multithread this??)

    // turn binaries into dockerimages

    // wait for thread A & thread B to finish..

    // scp dockerimages to instance

    // ssh into instance, start docker containers with new images

    // test?
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

fn ssh_and_run_initial_commands() {
    executor::spawn(Compat::new(async move {
        let result = ssh_and_run_initial_commands_async().await;
        match result {
            Ok(_) => info!("SSH success!"),
            Err(e) => info!("SSH error: {:?}", e),
        }
    }))
        .detach();

    loop {
        std::thread::sleep(std::time::Duration::from_secs(5));
        info!(".");
    }
}

async fn ssh_and_run_initial_commands_async() -> Result<(), OpenSshError> {

    let key_path = Path::new("~/Work/cyberlith/.vultr/vultrkey");

    let ssh_path = format!("ssh://root@{}", get_static_ip());

    let session = SessionBuilder::default()
        .known_hosts_check(KnownHosts::Accept)
        .keyfile(key_path)
        .connect(ssh_path)
        .await?;

    setup_iptables(&session).await?;
    setup_docker(&session).await?;

    session.close().await?;

    Ok(())
}

async fn setup_iptables(session: &Session) -> Result<(), OpenSshError> {

    info!("// allow established connections");
    run_ssh_command(&session, "sudo iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT").await?;

    info!("// allow ssh");
    run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport ssh -j ACCEPT").await?;

    info!("// allow loopback");
    run_ssh_command(&session, "sudo iptables -I INPUT 1 -i lo -j ACCEPT").await?;

    info!("// allow port 80 (content server)");
    run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport 80 -j ACCEPT").await?;

    info!("// allow port 14197 (orchestrator)");
    run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport 14197 -j ACCEPT").await?;

    info!("// allow port 14200 (session signal)");
    run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport 14200 -j ACCEPT").await?;

    info!("// allow port 14201 (session webrtc)");
    run_ssh_command(&session, "sudo iptables -A INPUT -p udp --dport 14201 -j ACCEPT").await?;

    info!("// allow port 14203 (world signal)");
    run_ssh_command(&session, "sudo iptables -A INPUT -p tcp --dport 14203 -j ACCEPT").await?;

    info!("// allow port 14204 (world webrtc)");
    run_ssh_command(&session, "sudo iptables -A INPUT -p udp --dport 14204 -j ACCEPT").await?;

    Ok(())
}

async fn setup_docker(session: &Session) -> Result<(), OpenSshError> {

    info!("// update");
    run_ssh_command(&session, "sudo apt-get update").await?;

    info!("// install dependencies");
    run_ssh_command(&session, "sudo apt-get install ca-certificates curl -y").await?;

    info!("// install keyring");
    run_ssh_command(&session, "sudo install -m 0755 -d /etc/apt/keyrings").await?;

    info!("// download GPG key and install");
    run_ssh_command(&session, "sudo curl -fsSL https://download.docker.com/linux/ubuntu/gpg -o /etc/apt/keyrings/docker.asc").await?;

    info!("// set permissions on keyring");
    run_ssh_command(&session, "sudo chmod a+r /etc/apt/keyrings/docker.asc").await?;

    info!("// add docker to apt sources?");
    run_ssh_raw_command(&session, "echo \"deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo \"$VERSION_CODENAME\") stable\" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null").await?;

    info!("// update");
    run_ssh_command(&session, "sudo apt-get update").await?;

    info!("// install docker packages");
    run_ssh_command(&session, "sudo apt-get install docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin -y").await?;

    info!("// add user to docker group");
    run_ssh_command(&session, "sudo usermod -aG docker $USER").await?;

    info!("// test that docker works without sudo");
    run_ssh_command(&session, "docker version").await?;


    Ok(())
}

async fn run_ssh_command(session: &Session, command_str: &str) -> Result<(), OpenSshError> {
    info!("-> {}", command_str);

    let commands: Vec<String> = command_str.split(" ").map(|thestr| thestr.to_string()).collect();

    let mut command = session.command(&commands[0]);
    for i in 1..commands.len() {
        command.arg(&commands[i]);
    }

    let output = command.output().await?;
    info!(
        "<- {}",
        String::from_utf8(output.stdout).expect("server output was not valid UTF-8")
    );

    Ok(())
}

async fn run_ssh_raw_command(session: &Session, command_str: &str) -> Result<(), OpenSshError> {
    info!("-> {}", command_str);

    let mut raw_command = session.raw_command(command_str);
    let output = raw_command.output().await?;
    info!(
        "<- {}",
        String::from_utf8(output.stdout).expect("server output was not valid UTF-8")
    );

    Ok(())
}