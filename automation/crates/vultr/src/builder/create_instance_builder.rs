use crate::{VultrApi, VultrError, VultrInstance, VultrInstanceRoot, VultrInstanceType};
use serde::Serialize;

#[derive(Debug, Clone)]
pub enum LinuxUserScheme {
    Root,
    Limited,
}

#[derive(Serialize, Debug)]
struct CreateInstanceConfig {
    region: String,
    plan: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    os_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ipxe_chain_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    iso_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    script_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    snapshot_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_ipv6: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    disable_public_ipv4: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attach_vpc: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    attach_vpc2: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sshkey_id: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backups: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    app_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    image_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ddos_protection: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    activation_email: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hostname: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    firewall_group_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reserved_ipv4: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_vpc: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    enable_vpc2: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_scheme: Option<String>,
}

/// Builder struct for creating instances.
///
/// A detailed documentation can be found at <https://www.vultr.com/api/#tag/instances/operation/create-instance>
pub struct CreateInstanceBuilder {
    api: VultrApi,
    config: CreateInstanceConfig,
}

impl CreateInstanceBuilder {
    pub fn new<S1, S2>(
        api: VultrApi,
        region_id: S1,
        plan_id: S2,
        instance_type: VultrInstanceType,
    ) -> Self
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let mut instancebuilder = CreateInstanceBuilder {
            api,
            config: CreateInstanceConfig {
                region: region_id.into(),
                plan: plan_id.into(),
                enable_ipv6: None,
                label: None,
                sshkey_id: None,
                backups: None,
                ddos_protection: None,
                activation_email: None,
                hostname: None,
                tags: None,
                os_id: None,
                iso_id: None,
                snapshot_id: None,
                app_id: None,
                ipxe_chain_url: None,
                script_id: None,
                disable_public_ipv4: None,
                attach_vpc: None,
                attach_vpc2: None,
                image_id: None,
                user_data: None,
                firewall_group_id: None,
                reserved_ipv4: None,
                enable_vpc: None,
                enable_vpc2: None,
                user_scheme: None,
            },
        };
        match instance_type {
            VultrInstanceType::OS(id) => instancebuilder.config.os_id = Some(id.to_string()),
            VultrInstanceType::ISO(id) => instancebuilder.config.iso_id = Some(id.to_string()),
            VultrInstanceType::Snapshot(id) => {
                instancebuilder.config.snapshot_id = Some(id.to_string())
            }
            VultrInstanceType::App(id) => instancebuilder.config.app_id = Some(id.to_string()),
            VultrInstanceType::Image(id) => instancebuilder.config.image_id = Some(id.to_string()),
        };
        instancebuilder
    }

    pub fn enable_ipv6(mut self, enable_ipv6: bool) -> Self {
        self.config.enable_ipv6 = Some(enable_ipv6);
        self
    }

    /// A user-supplied label for this instance.
    pub fn label<S>(mut self, label: S) -> Self
    where
        S: Into<String>,
    {
        self.config.label = Some(label.into());
        self
    }

    /// The SSH Key id to install on this instance.
    pub fn sshkey_id<S>(mut self, sshkey_id: S) -> Self
    where
        S: Into<String>,
    {
        let sshkey_id_str = sshkey_id.into();
        let mut sshkey_ids = Vec::new();
        sshkey_ids.push(sshkey_id_str);
        self.config.sshkey_id = Some(sshkey_ids);
        self
    }

    /// Enable automatic backups for the instance.
    pub fn backups(mut self, backups: bool) -> Self {
        let backups = match backups {
            true => "enabled",
            false => "disabled",
        }
        .to_string();
        self.config.backups = Some(backups);
        self
    }

    /// Enable DDoS protection (there is an additional charge for this).
    pub fn ddos_protection(mut self, ddos_protection: bool) -> Self {
        self.config.ddos_protection = Some(ddos_protection);
        self
    }

    /// Notify by email after deployment.
    pub fn activation_email(mut self, activation_email: bool) -> Self {
        self.config.activation_email = Some(activation_email);
        self
    }

    /// The hostname to use when deploying this instance.
    pub fn hostname<S>(mut self, hostname: S) -> Self
    where
        S: Into<String>,
    {
        self.config.hostname = Some(hostname.into());
        self
    }

    /// The URL location of the iPXE chainloader.
    pub fn ipxe_chain_url<S>(mut self, ipxe_chain_url: S) -> Self
    where
        S: Into<String>,
    {
        self.config.ipxe_chain_url = Some(ipxe_chain_url.into());
        self
    }

    pub fn script_id<S>(mut self, script_id: S) -> Self
    where
        S: Into<String>,
    {
        self.config.script_id = Some(script_id.into());
        self
    }

    /// Don't set up a public IPv4 address when IPv6 is enabled. Will not do anything unless enable_ipv6 is also true.
    pub fn disable_public_ipv4(mut self, disable_public_ipv4: bool) -> Self {
        self.config.disable_public_ipv4 = Some(disable_public_ipv4);
        self
    }

    /// An array of VPC IDs to attach to this Instance. This parameter takes precedence over enable_vpc. Please choose one parameter.
    pub fn attach_vpc(mut self, attach_vpc: Vec<String>) -> Self {
        self.config.attach_vpc = Some(attach_vpc);
        self
    }

    /// An array of VPC IDs to attach to this Instance. This parameter takes precedence over enable_vpc2. Please choose one parameter.
    pub fn attach_vpc2(mut self, attach_vpc2: Vec<String>) -> Self {
        self.config.attach_vpc2 = Some(attach_vpc2);
        self
    }

    /// The user-supplied, base64 encoded user data to attach to this instance.
    pub fn user_data<S>(mut self, user_data: S) -> Self
    where
        S: Into<String>,
    {
        self.config.user_data = Some(user_data.into());
        self
    }

    /// The Firewall Group id to attach to this Instance.
    pub fn firewall_group_id<S>(mut self, firewall_group_id: S) -> Self
    where
        S: Into<String>,
    {
        self.config.firewall_group_id = Some(firewall_group_id.into());
        self
    }

    /// ID of the floating IP to use as the main IP of this server.
    pub fn reserved_ipv4<S>(mut self, reserved_ipv4: S) -> Self
    where
        S: Into<String>,
    {
        self.config.reserved_ipv4 = Some(reserved_ipv4.into());
        self
    }

    /// If true, VPC support will be added to the new server.
    ///
    /// This parameter attaches a single VPC. When no VPC exists in the region, it will be automatically created.
    ///
    /// If there are multiple VPCs in the instance's region, use attach_vpc instead to specify a network.
    pub fn enable_vpc(mut self, enable_vpc: bool) -> Self {
        self.config.enable_vpc = Some(enable_vpc);
        self
    }

    /// If true, VPC 2.0 support will be added to the new server.
    ///
    /// This parameter attaches a single VPC 2.0 network. When no VPC 2.0 network exists in the region, it will be automatically created.
    ///
    /// If there are multiple VPC 2.0 networks in the instance's region, use attach_vpc2 instead to specify a network.
    pub fn enable_vpc2(mut self, enable_vpc2: bool) -> Self {
        self.config.enable_vpc2 = Some(enable_vpc2);
        self
    }

    /// Linux-only: The user scheme used for logging into this instance.
    /// By default, the "root" user is configured.
    /// Alternatively, a limited user with sudo permissions can be selected.
    pub fn user_scheme<S>(mut self, user_scheme: LinuxUserScheme) -> Self {
        match user_scheme {
            LinuxUserScheme::Root => self.config.user_scheme = Some("root".to_string()),
            LinuxUserScheme::Limited => self.config.user_scheme = Some("limited".to_string()),
        };
        self
    }

    /// Tags to apply to the instance
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.config.tags = Some(tags);
        self
    }

    #[cfg(feature = "blocking")]
    pub fn run(self) -> Result<VultrInstance, VultrError> {
        let url = format!("https://api.vultr.com/v2/instances");
        Ok(self
            .api
            .post(&url, self.config)?
            .json::<VultrInstanceRoot>()?
            .instance)
    }

    pub async fn run_async(self) -> Result<VultrInstance, VultrError> {
        let url = format!("https://api.vultr.com/v2/instances");
        Ok(self
            .api
            .post_async(&url, self.config)
            .await?
            .json::<VultrInstanceRoot>()
            .await?
            .instance)
    }
}
