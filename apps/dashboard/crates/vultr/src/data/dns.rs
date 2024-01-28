use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct VultrDomainsRoot {
    pub domains: Vec<VultrDomain>,
}

#[derive(Deserialize, Debug)]
pub struct VultrDomainRoot {
    pub domain: VultrDomain,
}

#[derive(Deserialize, Debug)]
pub struct VultrDomain {
    pub domain: String,
    pub date_created: String,
}

#[derive(Deserialize, Debug)]
pub struct VultrDomainRecordsRoot {
    pub records: Vec<VultrDomainRecord>,
}

#[derive(Deserialize, Debug)]
pub struct VultrDomainRecordRoot {
    pub record: VultrDomainRecord,
}

#[derive(Deserialize, Debug)]
pub struct VultrDomainRecord {
    pub id: String,
    #[serde(rename = "type")]
    pub record_type: String,
    pub name: String,
    pub data: String,
    pub priority: i32,
    pub ttl: u32,
}