use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct VultrAccountRoot {
    pub account: VultrAccount,
}

#[derive(Deserialize, Debug)]
pub struct VultrAccount {
    pub name: String,
    pub email: String,
    pub balance: f32,
    pub pending_charges: f32,
    pub last_payment_date: String,
    pub last_payment_amount: f32,
    pub acls: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct VultrRegionsRoot {
    pub regions: Vec<VultrRegion>,
}

#[derive(Deserialize, Debug)]
pub struct VultrRegion {
    pub id: String,
    pub city: String,
    pub country: String,
    pub continent: String,
    pub options: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct VultrPlansRoot {
    pub plans: Vec<VultrPlan>,
}

#[derive(Deserialize, Debug)]
pub struct VultrPlan {
    pub id: String,
    pub vcpu_count: u8,
    pub ram: u32,
    pub disk: f32,
    pub bandwidth: f32,
    pub monthly_cost: f32,
    #[serde(rename = "type")]
    pub plan_type: String,
    pub locations: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct VultrOSRoot {
    pub os: Vec<VultrOS>,
}

#[derive(Deserialize, Debug)]
pub struct VultrOS {
    pub id: u32,
    pub name: String,
    pub arch: String,
    pub family: String,
}

#[derive(Deserialize, Debug)]
pub struct VultrReservedIpsRoot {
    pub reserved_ips: Vec<VultrReservedIp>,
}

#[derive(Deserialize, Debug)]
pub struct VultrReservedIp {
    pub id: String,
    pub region: String,
    pub ip_type: String,
    pub subnet: String,
    pub subnet_size: u32,
    pub label: String,
    pub instance_id: String,
}

#[derive(Deserialize, Debug)]
pub struct VultrSSHKeyRoot {
    pub ssh_key: VultrSSHKey,
}

#[derive(Deserialize, Debug)]
pub struct VultrSSHKeysRoot {
    pub ssh_keys: Vec<VultrSSHKey>,
}

#[derive(Deserialize, Debug)]
pub struct VultrSSHKey {
    pub id: String,
    pub date_created: String,
    pub name: String,
    pub ssh_key: String,
}

#[derive(Deserialize, Debug)]
pub struct VultrInstanceRoot {
    pub instance: VultrInstance,
}

#[derive(Deserialize, Debug)]
pub struct VultrInstancesRoot {
    pub instances: Vec<VultrInstance>,
}

#[derive(Deserialize, Debug)]
pub struct VultrInstance {
    pub id: String,
    pub os: String,
    pub ram: f32,
    pub disk: f32,
    pub main_ip: String,
    pub vcpu_count: u32,
    pub region: String,
    pub plan: String,
    pub date_created: String,
    pub status: String,
    pub allowed_bandwidth: f32,
    pub netmask_v4: String,
    pub gateway_v4: String,
    pub power_status: String,
    pub server_status: String,
    pub v6_network: String,
    pub v6_main_ip: String,
    pub v6_network_size: u64,
    pub label: String,
    pub internal_ip: String,
    pub kvm: String,
    pub hostname: String,
    pub os_id: u32,
    pub app_id: u32,
    pub image_id: String,
    pub firewall_group_id: String,
    pub features: Vec<String>,
    pub tags: Vec<String>,
    pub user_scheme: String,
}