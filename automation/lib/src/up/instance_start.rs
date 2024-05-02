use logging::info;
use vultr::{VultrApi, VultrInstanceType};

use crate::{get_api_key, CliError};

pub async fn instance_start() -> Result<String, CliError> {
    let api_key = get_api_key();

    let api = VultrApi::new(api_key);

    let instances = api
        .get_instance_list_async()
        .await
        .map_err(|e| CliError::from(e))?;
    if instances.len() == 1 {
        let instance = instances.get(0).unwrap();
        return Ok(instance.id.clone());
    }
    if instances.len() > 1 {
        return Err(CliError::Message("Multiple instances running".to_string()));
    }

    // get region id
    let regions = api.get_regions_async().await?;
    let region = regions
        .iter()
        .find(|r| r.city == "Chicago" && r.country == "US" && r.continent == "North America")
        .ok_or(CliError::Message("No region found".to_string()))?;
    let region_id = region.id.clone();
    info!("found region id: {}", region_id);

    // get plan id
    let plans = api.get_plans_async().await?;
    let plan = plans
        .iter()
        .find(|p| {
            p.monthly_cost == 5.0
                && p.vcpu_count == 1
                && p.plan_type == "vc2"
                && p.ram == 1024
                && p.disk == 25.0
                && p.bandwidth == 1024.0
        })
        .ok_or(CliError::Message("No plan found".to_string()))?;
    let plan_id = plan.id.clone();
    info!("found plan id: {:?}", plan_id);

    // // get os id
    let oses = api.get_os_list_async().await?;
    let os = oses
        .iter()
        .find(|o| {
            o.family.contains("ubuntu") && o.arch == "x64" && o.name == "Ubuntu 22.04 LTS x64"
        })
        .ok_or(CliError::Message("No OS found".to_string()))?;
    let os_id = os.id;
    info!("found OS id: {:?}", os_id);

    // get ssh key id
    let ssh_keys = api.get_sshkey_list_async().await?;
    let ssh_key = ssh_keys
        .iter()
        .find(|k| k.name == "Primary")
        .ok_or(CliError::Message("No SSH key found".to_string()))?;
    let ssh_key_id = ssh_key.id.clone();
    info!("found ssh key id: {:?}", ssh_key_id);

    // get reserved ip id
    let reserved_ips = api.get_reserved_ip_list_async().await?;
    let reserved_ip = reserved_ips
        .iter()
        .find(|i| i.label == "Primary")
        .ok_or(CliError::Message("No reserved IP found".to_string()))?;
    let reserved_ip_id = reserved_ip.id.clone();
    info!("found reserved ip id: {:?}", reserved_ip_id);

    // get firewall group id
    let firewall_groups = api.get_firewall_group_list_async().await?;
    let firewall_group = firewall_groups
        .iter()
        .find(|g| g.description == "primary_firewall")
        .ok_or(CliError::Message("No firewall group found".to_string()))?;
    let firewall_group_id = firewall_group.id.clone();
    info!("found firewall group id: {:?}", firewall_group_id);

    // get ubuntu server iso id
    // let isos = api.get_iso_list()?;
    // let iso = isos
    //     .iter()
    //     .find(|i| i.filename == "ubuntu-22.04.3-live-server-amd64.iso")
    //     .ok_or(CliError::Message("No ISO found".to_string()))?;
    // let iso_id = iso.id.clone();
    // info!("found iso id: {:?}", iso_id);

    // create instance
    let instance_opt;
    loop {
        let instance_result = api
            .create_instance(&region_id, &plan_id, VultrInstanceType::OS(os_id))
            .hostname("primaryserver")
            .label("Primary Server")
            .reserved_ipv4(&reserved_ip_id)
            .sshkey_id(&ssh_key_id)
            .firewall_group_id(&firewall_group_id)
            .enable_ipv6(false)
            .backups(false)
            .ddos_protection(false)
            .activation_email(false)
            .run_async()
            .await;
        match instance_result {
            Ok(instance) => {
                instance_opt = Some(instance);
                break;
            }
            Err(err) => {
                info!("error creating instance: {:?}", err);
                info!("retrying after 5 seconds..");
                executor::smol::Timer::after(std::time::Duration::from_secs(5)).await;
                continue;
            }
        }
    }

    let instance = instance_opt.unwrap();
    Ok(instance.id)
}
