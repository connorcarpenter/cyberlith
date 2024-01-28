use log::info;

use vultr::{VultrApi, VultrError, VultrInstanceType};

use crate::get_api_key;

pub fn up() {
    info!("Starting vultr instance");
    let result = start_instance();
    match result {
        Ok(instance_id) => info!("Instance started! id is '{}'", instance_id),
        Err(e) => info!("Error starting instance: {:?}", e),
    }
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

    // get os id
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

    let instance = api
        .create_instance(
            &region_id,
            &plan_id,
            VultrInstanceType::OS(os_id),
        )
        .hostname("primaryserver")
        .label("Primary Server")

        .sshkey_id(&ssh_key.id)
        .enable_ipv6(false)
        .backups(false)
        .ddos_protection(false)
        .activation_email(false)

        .run()?;

    Ok(instance.id)

    // let account = api.get_account_info()?;
    // info!("ACCOUNT: {:?}", account);

    // let new_domain = api.create_dns_domain(domain, None, false)?;
    // println!("CREATED DOMAIN: {:?}", new_domain);
    //
    // let old_domain = api.get_dns_domain(domain)?;
    // println!("GET DOMAIN: {:?}", old_domain);
    //
    // let record = api.create_dns_domain_record(domain, "A", "www", "10.0.0.8", None, None)?;
    // println!("RECORD CREATED: {:?}", record);
    //
    // let records = api.get_dns_domain_records(domain)?;
    // println!("RECORDS: {:?}", records);
    //
    // let record = api.delete_dns_domain_record(domain, &record.id);
    // println!("RECORD DELETED: {:?}", record);
    //
    // let domains = api.get_dns_domain_list()?;
    // println!("DOMAIN LIST: {:?}", domains);

    // let old_domain = api.delete_dns_domain(domain)?;
    // println!("DEL DOMAIN: {:?}", old_domain);

    // let regions = api.get_regions()?;
    // info!("REGIONS: {:?}", regions);

    // let plans = api.get_plans()?;
    // info!("PLANS: {:?}", plans);

    // let mut os = api.get_os_list()?;
    // let ubuntu_list: Vec<VultrOS> = os
    //     .drain(..)
    //     .filter(|item| item.family.contains("ubuntu"))
    //     .collect();
    // info!("UBUNTU LIST: {:?}", ubuntu_list);

    // let ssh_key = api.create_sshkey("test", "xxx")?;
    // info!("SSH KEY CREATED: {:?}", ssh_key);
    //
    // let ssh_key = api.get_sshkey(ssh_key.id)?;
    // info!("SSH KEY: {:?}", ssh_key);

    // let ssh_keys = api.get_sshkey_list()?;
    // info!("SSH KEYS: {:?}", ssh_keys);
}