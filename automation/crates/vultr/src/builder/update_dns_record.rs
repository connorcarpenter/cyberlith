use crate::{VultrApi, VultrError};
use serde::Serialize;

pub struct VultrUpdateDnsRecordBuilder {
    api: VultrApi,
    domainname: String,
    record_id: String,
    config: UpdateDnsRecordConfig,
}

#[derive(Serialize, Debug)]
struct UpdateDnsRecordConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ttl: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<u64>,
}

impl VultrUpdateDnsRecordBuilder {
    pub fn new<S1: Into<String>, S2: Into<String>>(
        api: &VultrApi,
        domainname: S1,
        record_id: S2,
    ) -> Self {
        VultrUpdateDnsRecordBuilder {
            api: api.clone(),
            domainname: domainname.into(),
            record_id: record_id.into(),
            config: UpdateDnsRecordConfig {
                name: None,
                data: None,
                ttl: None,
                priority: None,
            },
        }
    }

    pub fn name<S: Into<String>>(mut self, b: S) -> VultrUpdateDnsRecordBuilder {
        self.config.name = Some(b.into());
        self
    }

    pub fn data<S: Into<String>>(mut self, data: S) -> VultrUpdateDnsRecordBuilder {
        self.config.data = Some(data.into());
        self
    }

    pub fn ttl(mut self, ttl: u64) -> VultrUpdateDnsRecordBuilder {
        self.config.ttl = Some(ttl);
        self
    }

    pub fn priority(mut self, priority: u64) -> VultrUpdateDnsRecordBuilder {
        self.config.priority = Some(priority);
        self
    }

    #[cfg(feature = "blocking")]
    pub fn run(self) -> Result<(), VultrError> {
        let url = format!(
            "https://api.vultr.com/v2/domains/{domainname}/records/{record_id}",
            domainname = self.domainname,
            record_id = self.record_id
        );
        self.api.patch(&url, self.config)?.error_for_status()?;
        Ok(())
    }

    pub async fn run_async(self) -> Result<(), VultrError> {
        let url = format!(
            "https://api.vultr.com/v2/domains/{domainname}/records/{record_id}",
            domainname = self.domainname,
            record_id = self.record_id
        );
        self.api
            .patch_async(&url, self.config)
            .await?
            .error_for_status()?;
        Ok(())
    }
}
