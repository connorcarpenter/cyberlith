use std::fmt::Display;

use serde::{Deserialize, Serialize};

use db::{DbRowKey, DbRowValue, DbTableKey};

// table key
pub struct Certs;

impl DbTableKey for Certs {
    type Key = CertId;
    type Value = Cert;

    fn repo_name() -> &'static str {
        "cyberlith_certs"
    }
}

// user id
#[derive(Serialize, Deserialize, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub struct CertId {
    id: u64,
}

impl DbRowKey for CertId {}

impl CertId {
    pub fn new(id: u64) -> Self {
        Self { id }
    }
}

impl From<u64> for CertId {
    fn from(id: u64) -> Self {
        Self { id }
    }
}

impl Into<u64> for CertId {
    fn into(self) -> u64 {
        self.id
    }
}

// cert type ... I know the name doesn't make sense!
#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq)]
pub enum CertType {
    Cert,
    Account,
}

impl Display for CertType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CertType::Cert => write!(f, "Cert"),
            CertType::Account => write!(f, "Account"),
        }
    }
}

// cert
#[derive(Serialize, Deserialize, Clone)]
pub struct Cert {
    id: Option<u64>,
    cert_type: CertType,
    name: String,
    bytes: Vec<u8>,
}

impl Cert {}

impl DbRowValue for Cert {
    type Key = CertId;

    fn get_key(&self) -> <Self as DbRowValue>::Key {
        <Self as DbRowValue>::Key::from(self.id.unwrap())
    }

    fn set_key(&mut self, key: <Self as DbRowValue>::Key) {
        self.id = Some(key.into());
    }

    fn get_file_name(&self) -> String {
        self.id.unwrap().to_string()
    }

    fn get_insert_commit_message(&self) -> String {
        match self.cert_type {
            CertType::Cert => format!("adding: [Cert: (id: {})]", self.id.unwrap()),
            CertType::Account => format!("adding: [Account: (id: {})]", self.id.unwrap()),
        }
    }

    fn get_update_commit_message(&self) -> String {
        match self.cert_type {
            CertType::Cert => format!("updating: [Cert: (id: {})]", self.id.unwrap()),
            CertType::Account => format!("updating: [Account: (id: {})]", self.id.unwrap()),
        }
    }
}

impl Cert {
    pub fn new(name: String, cert_type: CertType, bytes: Vec<u8>) -> Self {
        Self {
            id: None,
            name,
            cert_type,
            bytes,
        }
    }

    pub fn cert_type(&self) -> CertType {
        self.cert_type
    }

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }

    pub fn set_bytes(&mut self, bytes: Vec<u8>) {
        self.bytes = bytes;
    }
}
