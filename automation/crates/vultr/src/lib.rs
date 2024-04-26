//! Client library for the <https://www.vultr.com/> API which
//! is documented at <https://www.vultr.com/api/>
//!
//! # Example blocking
//! It needs to have the feature "blocking" enabled.
//! ```toml
//! vultr = { version = "*", features = ["blocking"] }
//! ```
//! ```ignore
//! use vultr::VultrApi;
//! use vultr::VultrError;
//!
//! fn main() -> Result<(), VultrError> {
//!     let api = VultrApi::new("<KEY>");
//!     let account = api.get_account_info()?;
//!     println!("ACCOUNT: {:?}", account);
//!
//!     let regions = api.get_regions()?;
//!     println!("REGIONS: {:?}", regions);
//!         
//!     let plans = api.get_plans()?;
//!     println!("PLANS: {:?}", plans);
//!     
//!     let os = api.get_os_list()?;
//!     println!("OS: {:?}", os);
//!     Ok(())
//! }
//! ```
//!
//! # Example async
//! ```toml
//! vultr = { version = "*" }
//! ```
//! ```no_run
//! use vultr::VultrApi;
//! use vultr::VultrError;
//!
//! #[async_std::main]
//! async fn main() -> Result<(), VultrError> {
//!     let api = VultrApi::new("<KEY>");
//!     let account = api.get_account_info_async().await?;
//!     println!("ACCOUNT: {:?}", account);
//!     Ok(())
//! }
//! ```
//! ## Features
//! * "default" - use nativetls
//! * "default-rustls" - use rusttls
//! * "blocking" - enable blocking api
//! * "rustls" - enable rustls for reqwest
//! * "nativetls" - add support for nativetls DEFAULT
//! * "gzip" - enable gzip in reqwest
//! * "brotli" - enable brotli in reqwest
//! * "deflate" - enable deflate in reqwest

mod api_error;
mod builder;
mod data;
mod instance_type;
mod vultr_error;

use std::collections::HashMap;

use api_error::VultrApiError;
use serde::Serialize;
use serde_json::json;

pub use builder::{
    create_instance_builder::CreateInstanceBuilder, create_instance_builder::LinuxUserScheme,
    update_dns_record::VultrUpdateDnsRecordBuilder,
};
pub use data::{
    dns::{
        VultrDomain, VultrDomainRecord, VultrDomainRecordRoot, VultrDomainRecordsRoot,
        VultrDomainRoot, VultrDomainsRoot,
    },
    instance::{
        VultrAccount, VultrAccountRoot, VultrFirewallGroup, VultrFirewallGroupsRoot, VultrInstance,
        VultrInstanceRoot, VultrInstancesRoot, VultrIso, VultrIsosRoot, VultrOS, VultrOSRoot,
        VultrPlan, VultrPlansRoot, VultrRegion, VultrRegionsRoot, VultrReservedIp,
        VultrReservedIpsRoot, VultrSSHKey, VultrSSHKeyRoot, VultrSSHKeysRoot,
    },
};
pub use instance_type::VultrInstanceType;
pub use vultr_error::VultrError;

#[derive(Clone)]
pub struct VultrApi {
    token: String,
}

impl<'a> VultrApi {
    pub fn new<S>(token: S) -> VultrApi
    where
        S: Into<String>,
    {
        VultrApi {
            token: token.into(),
        }
    }

    async fn get_async(&self, url: &str) -> Result<reqwest::Response, VultrError> {
        let client = reqwest::Client::new();
        let resp = client
            .get(url)
            .bearer_auth(&self.token)
            .send()
            .await
            .map_err(|e| VultrError::Reqwest(e))?;
        let status = resp.status();
        if status.is_client_error() {
            let result: VultrApiError = resp.json().await?;
            Err(VultrError::VultrApi(result.error))
        } else {
            Ok(resp.error_for_status()?)
        }
    }

    #[cfg(feature = "blocking")]
    fn get(&self, url: &str) -> Result<reqwest::blocking::Response, VultrError> {
        let client = reqwest::blocking::Client::new();
        let resp = client.get(url).bearer_auth(&self.token).send()?;
        let status = resp.status();
        if status.is_client_error() {
            let result: VultrApiError = resp.json()?;
            Err(VultrError::VultrApi(result.error))
        } else {
            Ok(resp.error_for_status()?)
        }
    }

    async fn post_async<T>(&self, url: &str, json: T) -> Result<reqwest::Response, VultrError>
    where
        T: Serialize + Sized,
    {
        let client = reqwest::Client::new();
        let resp = client
            .post(url)
            .bearer_auth(&self.token)
            .json(&json)
            .send()
            .await?;
        let status = resp.status();
        if status.is_client_error() {
            let result: VultrApiError = resp.json().await?;
            Err(VultrError::VultrApi(result.error))
        } else {
            Ok(resp.error_for_status()?)
        }
    }

    #[cfg(feature = "blocking")]
    fn post<T>(&self, url: &str, json: T) -> Result<reqwest::blocking::Response, VultrError>
    where
        T: Serialize + Sized,
    {
        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(url)
            .bearer_auth(&self.token)
            .json(&json)
            .send()?;
        let status = resp.status();
        if status.is_client_error() {
            let result: VultrApiError = resp.json()?;
            Err(VultrError::VultrApi(result.error))
        } else {
            Ok(resp.error_for_status()?)
        }
    }

    async fn patch_async<T>(&self, url: &str, json: T) -> Result<reqwest::Response, VultrError>
    where
        T: Serialize + Sized,
    {
        let client = reqwest::Client::new();
        let resp = client
            .patch(url)
            .bearer_auth(&self.token)
            .json(&json)
            .send()
            .await?;
        let status = resp.status();
        if status.is_client_error() {
            let result: VultrApiError = resp.json().await?;
            Err(VultrError::VultrApi(result.error))
        } else {
            Ok(resp.error_for_status()?)
        }
    }

    #[cfg(feature = "blocking")]
    fn patch<T>(&self, url: &str, json: T) -> Result<reqwest::blocking::Response, VultrError>
    where
        T: Serialize + Sized,
    {
        let client = reqwest::blocking::Client::new();
        let resp = client
            .patch(url)
            .bearer_auth(&self.token)
            .json(&json)
            .send()?;
        let status = resp.status();
        if status.is_client_error() {
            let result: VultrApiError = resp.json()?;
            Err(VultrError::VultrApi(result.error))
        } else {
            Ok(resp.error_for_status()?)
        }
    }

    async fn delete_async(&self, url: &str) -> Result<reqwest::Response, VultrError> {
        let client = reqwest::Client::new();
        let resp = client.delete(url).bearer_auth(&self.token).send().await?;
        let status = resp.status();
        if status.is_client_error() {
            let result: VultrApiError = resp.json().await?;
            Err(VultrError::VultrApi(result.error))
        } else {
            Ok(resp.error_for_status()?)
        }
    }

    #[cfg(feature = "blocking")]
    fn delete(&self, url: &str) -> Result<reqwest::blocking::Response, VultrError> {
        let client = reqwest::blocking::Client::new();
        let resp = client.delete(url).bearer_auth(&self.token).send()?;
        let status = resp.status();
        if status.is_client_error() {
            let result: VultrApiError = resp.json()?;
            Err(VultrError::VultrApi(result.error))
        } else {
            Ok(resp.error_for_status()?)
        }
    }

    pub async fn get_account_info_async(&self) -> Result<VultrAccount, VultrError> {
        Ok(self
            .get_async("https://api.vultr.com/v2/account")
            .await?
            .json::<VultrAccountRoot>()
            .await
            .map_err(|e| VultrError::Reqwest(e))?
            .account)
    }

    #[cfg(feature = "blocking")]
    pub fn get_account_info(&self) -> Result<VultrAccount, VultrError> {
        Ok(self
            .get("https://api.vultr.com/v2/account")?
            .json::<VultrAccountRoot>()?
            .account)
    }

    pub async fn get_dns_domain_list_async(&self) -> Result<Vec<VultrDomain>, VultrError> {
        Ok(self
            .get_async("https://api.vultr.com/v2/domains")
            .await?
            .json::<VultrDomainsRoot>()
            .await?
            .domains)
    }

    #[cfg(feature = "blocking")]
    pub fn get_dns_domain_list(&self) -> Result<Vec<VultrDomain>, VultrError> {
        Ok(self
            .get("https://api.vultr.com/v2/domains")?
            .json::<VultrDomainsRoot>()?
            .domains)
    }

    pub async fn get_dns_domain_async<S>(&self, domain: S) -> Result<VultrDomain, VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/domains/{}", domain.into());
        Ok(self
            .get_async(&url)
            .await?
            .json::<VultrDomainRoot>()
            .await?
            .domain)
    }

    #[cfg(feature = "blocking")]
    pub fn get_dns_domain<S>(&self, domain: S) -> Result<VultrDomain, VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/domains/{}", domain.into());
        Ok(self.get(&url)?.json::<VultrDomainRoot>()?.domain)
    }

    pub async fn delete_dns_domain_async<S>(&self, domain: S) -> Result<(), VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/domains/{}", domain.into());
        self.delete_async(&url).await?;
        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub fn delete_dns_domain<S>(&self, domain: S) -> Result<(), VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/domains/{}", domain.into());
        self.delete(&url)?;
        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub fn create_dns_domain<S>(
        &self,
        domain: S,
        ip: Option<String>,
        dns_sec: bool,
    ) -> Result<VultrDomain, VultrError>
    where
        S: Into<String>,
    {
        let mut map = HashMap::new();
        map.insert("domain", domain.into());
        if let Some(ip) = ip {
            map.insert("ip", ip);
        }
        map.insert(
            "dns_sec",
            if dns_sec {
                String::from("enabled")
            } else {
                String::from("disabled")
            },
        );

        let url = "https://api.vultr.com/v2/domains";
        Ok(self.post(&url, map)?.json::<VultrDomainRoot>()?.domain)
    }

    pub async fn create_dns_domain_async<S>(
        &self,
        domain: S,
        ip: Option<String>,
        dns_sec: bool,
    ) -> Result<VultrDomain, VultrError>
    where
        S: Into<String>,
    {
        let mut map = HashMap::new();
        map.insert("domain", domain.into());
        if let Some(ip) = ip {
            map.insert("ip", ip);
        }
        map.insert(
            "dns_sec",
            if dns_sec {
                String::from("enabled")
            } else {
                String::from("disabled")
            },
        );

        let url = "https://api.vultr.com/v2/domains";
        Ok(self
            .post_async(&url, map)
            .await?
            .json::<VultrDomainRoot>()
            .await?
            .domain)
    }

    #[cfg(feature = "blocking")]
    pub fn create_dns_domain_record<S1, S2, S3, S4>(
        &self,
        domain: S1,
        record_type: S2,
        name: S3,
        ip: S4,
        ttl: Option<u32>,
        priority: Option<u32>,
    ) -> Result<VultrDomainRecord, VultrError>
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    {
        let mut map: HashMap<&str, String> = HashMap::new();
        map.insert("type", record_type.into());
        map.insert("name", name.into());
        map.insert("data", ip.into());
        if let Some(ttl) = ttl {
            map.insert("ttl", ttl.to_string());
        }
        if let Some(priority) = priority {
            let priority = priority.to_string();
            map.insert("priority", priority);
        }

        let url = format!("https://api.vultr.com/v2/domains/{}/records", domain.into());
        Ok(self
            .post(&url, map)?
            .json::<VultrDomainRecordRoot>()?
            .record)
    }

    pub async fn create_dns_domain_record_async<S1, S2, S3, S4>(
        &self,
        domain: S1,
        record_type: S2,
        name: S3,
        ip: S4,
        ttl: Option<u32>,
        priority: Option<u32>,
    ) -> Result<VultrDomainRecord, VultrError>
    where
        S1: Into<String>,
        S2: Into<String>,
        S3: Into<String>,
        S4: Into<String>,
    {
        let mut map: HashMap<&str, String> = HashMap::new();
        map.insert("type", record_type.into());
        map.insert("name", name.into());
        map.insert("data", ip.into());
        if let Some(ttl) = ttl {
            map.insert("ttl", ttl.to_string());
        }
        if let Some(priority) = priority {
            let priority = priority.to_string();
            map.insert("priority", priority);
        }

        let url = format!("https://api.vultr.com/v2/domains/{}/records", domain.into());
        Ok(self
            .post_async(&url, map)
            .await?
            .json::<VultrDomainRecordRoot>()
            .await?
            .record)
    }

    pub fn update_dns_record<S1: Into<String>, S2: Into<String>>(
        &self,
        domainname: S1,
        record_id: S2,
    ) -> VultrUpdateDnsRecordBuilder {
        VultrUpdateDnsRecordBuilder::new(self, domainname, record_id)
    }

    #[cfg(feature = "blocking")]
    pub fn get_dns_domain_records<S>(&self, domain: S) -> Result<Vec<VultrDomainRecord>, VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/domains/{}/records", domain.into());
        let resp: VultrDomainRecordsRoot = self.get(&url)?.json()?;
        Ok(resp.records)
    }

    pub async fn get_dns_domain_records_async<S>(
        &self,
        domain: S,
    ) -> Result<Vec<VultrDomainRecord>, VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/domains/{}/records", domain.into());
        let resp: VultrDomainRecordsRoot = self.get_async(&url).await?.json().await?;
        Ok(resp.records)
    }

    #[cfg(feature = "blocking")]
    pub fn delete_dns_domain_record<S1, S2>(
        &self,
        domain: S1,
        record_id: S2,
    ) -> Result<(), VultrError>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let url = format!(
            "https://api.vultr.com/v2/domains/{}/records/{}",
            domain.into(),
            record_id.into(),
        );
        self.delete(&url)?;
        Ok(())
    }

    pub async fn delete_dns_domain_record_async<S1, S2>(
        &self,
        domain: S1,
        record_id: S2,
    ) -> Result<(), VultrError>
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        let url = format!(
            "https://api.vultr.com/v2/domains/{}/records/{}",
            domain.into(),
            record_id.into(),
        );
        self.delete_async(&url).await?;
        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub fn get_plans(&self) -> Result<Vec<VultrPlan>, VultrError> {
        let url = format!("https://api.vultr.com/v2/plans");
        Ok(self.get(&url)?.json::<VultrPlansRoot>()?.plans)
    }

    pub async fn get_plans_async(&self) -> Result<Vec<VultrPlan>, VultrError> {
        let url = format!("https://api.vultr.com/v2/plans");
        Ok(self
            .get_async(&url)
            .await?
            .json::<VultrPlansRoot>()
            .await?
            .plans)
    }

    #[cfg(feature = "blocking")]
    pub fn get_regions(&self) -> Result<Vec<VultrRegion>, VultrError> {
        let url = format!("https://api.vultr.com/v2/regions");
        Ok(self.get(&url)?.json::<VultrRegionsRoot>()?.regions)
    }

    pub async fn get_regions_async(&self) -> Result<Vec<VultrRegion>, VultrError> {
        let url = format!("https://api.vultr.com/v2/regions");
        Ok(self
            .get_async(&url)
            .await?
            .json::<VultrRegionsRoot>()
            .await?
            .regions)
    }

    #[cfg(feature = "blocking")]
    pub fn get_reserved_ip_list(&self) -> Result<Vec<VultrReservedIp>, VultrError> {
        let url = format!("https://api.vultr.com/v2/reserved-ips");
        Ok(self.get(&url)?.json::<VultrReservedIpsRoot>()?.reserved_ips)
    }

    pub async fn get_reserved_ip_list_async(&self) -> Result<Vec<VultrReservedIp>, VultrError> {
        let url = format!("https://api.vultr.com/v2/reserved-ips");
        Ok(self
            .get_async(&url)
            .await?
            .json::<VultrReservedIpsRoot>()
            .await?
            .reserved_ips)
    }

    #[cfg(feature = "blocking")]
    pub fn get_firewall_group_list(&self) -> Result<Vec<VultrFirewallGroup>, VultrError> {
        let url = format!("https://api.vultr.com/v2/firewalls");
        Ok(self
            .get(&url)?
            .json::<VultrFirewallGroupsRoot>()?
            .firewall_groups)
    }

    pub async fn get_firewall_group_list_async(
        &self,
    ) -> Result<Vec<VultrFirewallGroup>, VultrError> {
        let url = format!("https://api.vultr.com/v2/firewalls");
        Ok(self
            .get_async(&url)
            .await?
            .json::<VultrFirewallGroupsRoot>()
            .await?
            .firewall_groups)
    }

    #[cfg(feature = "blocking")]
    pub fn get_iso_list(&self) -> Result<Vec<VultrIso>, VultrError> {
        let url = format!("https://api.vultr.com/v2/iso");
        Ok(self.get(&url)?.json::<VultrIsosRoot>()?.isos)
    }

    pub async fn get_iso_list_async(&self) -> Result<Vec<VultrIso>, VultrError> {
        let url = format!("https://api.vultr.com/v2/iso");
        Ok(self
            .get_async(&url)
            .await?
            .json::<VultrIsosRoot>()
            .await?
            .isos)
    }

    #[cfg(feature = "blocking")]
    pub fn get_os_list(&self) -> Result<Vec<VultrOS>, VultrError> {
        let url = format!("https://api.vultr.com/v2/os");
        Ok(self.get(&url)?.json::<VultrOSRoot>()?.os)
    }

    pub async fn get_os_list_async(&self) -> Result<Vec<VultrOS>, VultrError> {
        let url = format!("https://api.vultr.com/v2/os");
        Ok(self.get_async(&url).await?.json::<VultrOSRoot>().await?.os)
    }

    #[cfg(feature = "blocking")]
    pub fn get_sshkey_list(&self) -> Result<Vec<VultrSSHKey>, VultrError> {
        let url = format!("https://api.vultr.com/v2/ssh-keys");
        Ok(self.get(&url)?.json::<VultrSSHKeysRoot>()?.ssh_keys)
    }

    pub async fn get_sshkey_list_async(&self) -> Result<Vec<VultrSSHKey>, VultrError> {
        let url = format!("https://api.vultr.com/v2/ssh-keys");
        Ok(self
            .get_async(&url)
            .await?
            .json::<VultrSSHKeysRoot>()
            .await?
            .ssh_keys)
    }

    #[cfg(feature = "blocking")]
    pub fn get_sshkey<S>(&self, key_id: S) -> Result<VultrSSHKey, VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/ssh-keys/{}", key_id.into());
        Ok(self.get(&url)?.json::<VultrSSHKeyRoot>()?.ssh_key)
    }

    pub async fn get_sshkey_async<S>(&self, key_id: S) -> Result<VultrSSHKey, VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/ssh-keys/{}", key_id.into());
        Ok(self
            .get_async(&url)
            .await?
            .json::<VultrSSHKeyRoot>()
            .await?
            .ssh_key)
    }

    #[cfg(feature = "blocking")]
    pub fn create_sshkey<S>(&self, name: S, ssh_key: S) -> Result<VultrSSHKey, VultrError>
    where
        S: Into<String>,
    {
        let mut map: HashMap<&str, String> = HashMap::new();
        map.insert("name", name.into());
        map.insert("ssh_key", ssh_key.into());

        let url = format!("https://api.vultr.com/v2/ssh-keys");
        Ok(self.post(&url, map)?.json::<VultrSSHKeyRoot>()?.ssh_key)
    }

    pub async fn create_sshkey_async<S>(
        &self,
        name: S,
        ssh_key: S,
    ) -> Result<VultrSSHKey, VultrError>
    where
        S: Into<String>,
    {
        let mut map: HashMap<&str, String> = HashMap::new();
        map.insert("name", name.into());
        map.insert("ssh_key", ssh_key.into());

        let url = format!("https://api.vultr.com/v2/ssh-keys");
        Ok(self
            .post_async(&url, map)
            .await?
            .json::<VultrSSHKeyRoot>()
            .await?
            .ssh_key)
    }

    #[cfg(feature = "blocking")]
    pub fn delete_sshkey<S>(&self, key_id: S) -> Result<(), VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/ssh-keys/{}", key_id.into());
        self.delete(&url)?;
        Ok(())
    }

    pub async fn delete_sshkey_async<S>(&self, key_id: S) -> Result<(), VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/ssh-keys/{}", key_id.into());
        self.delete_async(&url).await?;
        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub fn get_instance_list(&self) -> Result<Vec<VultrInstance>, VultrError> {
        let url = format!("https://api.vultr.com/v2/instances");
        let resp: VultrInstancesRoot = self.get(&url)?.json()?;
        Ok(resp.instances)
    }

    pub async fn get_instance_list_async(&self) -> Result<Vec<VultrInstance>, VultrError> {
        let url = format!("https://api.vultr.com/v2/instances");
        let resp: VultrInstancesRoot = self.get_async(&url).await?.json().await?;
        Ok(resp.instances)
    }

    #[cfg(feature = "blocking")]
    pub fn get_instance<S>(&self, instance_id: S) -> Result<VultrInstance, VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/instances/{}", instance_id.into());
        Ok(self.get(&url)?.json::<VultrInstanceRoot>()?.instance)
    }

    pub async fn get_instance_async<S>(&self, instance_id: S) -> Result<VultrInstance, VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/instances/{}", instance_id.into());
        Ok(self
            .get_async(&url)
            .await?
            .json::<VultrInstanceRoot>()
            .await?
            .instance)
    }

    /// More information at <https://www.vultr.com/api/#tag/instances/operation/create-instance>
    pub fn create_instance<S1, S2>(
        &self,
        region_id: S1,
        plan_id: S2,
        instance_type: VultrInstanceType,
    ) -> CreateInstanceBuilder
    where
        S1: Into<String>,
        S2: Into<String>,
    {
        CreateInstanceBuilder::new(self.clone(), region_id, plan_id, instance_type)
    }

    #[cfg(feature = "blocking")]
    pub fn stop_instance<S>(&self, instance_ids: Vec<S>) -> Result<(), VultrError>
    where
        S: Into<String>,
    {
        let mut vec: Vec<String> = Vec::new();

        for instance_id in instance_ids {
            vec.push(instance_id.into());
        }

        self.post(
            "https://api.vultr.com/v2/instances/halt",
            json!({"instance_ids":vec}),
        )?;
        Ok(())
    }

    pub async fn stop_instance_async<S, T>(&self, instance_ids: S) -> Result<(), VultrError>
    where
        S: IntoIterator<Item = T>,
        T: AsRef<str>,
    {
        let vec: Vec<String> = instance_ids
            .into_iter()
            .map(|item| item.as_ref().to_string())
            .collect();
        self.post_async(
            "https://api.vultr.com/v2/instances/halt",
            json!({"instance_ids":vec}),
        )
        .await?;
        Ok(())
    }

    #[cfg(feature = "blocking")]
    pub fn delete_instance<S>(&self, instance_id: S) -> Result<(), VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/instances/{}", instance_id.into());
        self.delete(&url)?;
        Ok(())
    }

    pub async fn delete_instance_async<S>(&self, instance_id: S) -> Result<(), VultrError>
    where
        S: Into<String>,
    {
        let url = format!("https://api.vultr.com/v2/instances/{}", instance_id.into());
        self.delete_async(&url).await?;
        Ok(())
    }
}
