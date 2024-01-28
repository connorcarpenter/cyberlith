use std::path::Path;

use log::info;
use vultr::{VultrApi, VultrError, VultrInstanceType};
use openssh::{KnownHosts, SessionBuilder, Error as OpenSshError};
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

    // ssh into instance, set up iptables & docker

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
    // let oses = api.get_os_list()?;
    // let os = oses
    //     .iter()
    //     .find(|o| o.family.contains("ubuntu") && o.arch == "x64" && o.name == "Ubuntu 22.04 LTS x64")
    //     .ok_or(VultrError::Dashboard("No OS found".to_string()))?;
    // let os_id = os.id;
    // info!("found OS id: {:?}", os_id);

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
    let isos = api.get_iso_list()?;
    let iso = isos
        .iter()
        .find(|i| i.filename == "ubuntu-22.04.3-live-server-amd64.iso")
        .ok_or(VultrError::Dashboard("No ISO found".to_string()))?;
    let iso_id = iso.id.clone();
    info!("found iso id: {:?}", iso_id);

    // create instance
    let instance = api
        .create_instance(
            &region_id,
            &plan_id,
            VultrInstanceType::ISO(iso_id),
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

fn ssh() {
    executor::spawn(Compat::new(async move {
        let result = ssh_impl().await;
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

async fn ssh_impl() -> Result<(), OpenSshError> {

    let key_path = Path::new("~/Work/cyberlith/.vultr/vultrkey");

    let ssh_path = format!("ssh://root@{}", get_static_ip());

    let session = SessionBuilder::default()
        .known_hosts_check(KnownHosts::Add)
        .keyfile(key_path)
        .connect(ssh_path)
        .await?;

    info!("hello?");

    let ls = session.command("ls").output().await?;
    info!(
        "{}",
        String::from_utf8(ls.stdout).expect("server output was not valid UTF-8")
    );

    let whoami = session.command("whoami").output().await?;
    info!(
        "{}",
        String::from_utf8(whoami.stdout).expect("server output was not valid UTF-8")
    );

    session.close().await?;

    info!("closing session");

    Ok(())
}