use std::{
    collections::HashMap,
    io::ErrorKind,
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use base64::prelude::*;
use blocking::unblock;
use ring::digest::{Context, SHA256};
use rustls_acme::{AccountCache, CertCache};

use acme_cert_db::{Cert, CertId, CertType, DatabaseManager};

pub struct GitCache {
    inner: Arc<RwLock<DatabaseManager>>,
    key_str_to_id_map: Arc<RwLock<HashMap<String, CertId>>>,
}

impl GitCache {
    pub fn new() -> Self {
        let inner = DatabaseManager::init();

        let key_str_to_id_map = Self::init_map(&inner);

        Self {
            inner: Arc::new(RwLock::new(inner)),
            key_str_to_id_map: Arc::new(RwLock::new(key_str_to_id_map)),
        }
    }

    fn init_map(dbm: &DatabaseManager) -> HashMap<String, CertId> {
        let mut output = HashMap::new();

        for (cert_id, cert) in dbm.list_certs() {
            let name = cert.name();
            output.insert(name, *cert_id);
        }

        output
    }

    async fn read_if_exist(&self, key_str: String) -> Result<Option<Vec<u8>>, std::io::Error> {
        let key_str_to_id_map = self.key_str_to_id_map.clone();
        let inner = self.inner.clone();
        unblock(move || {
            let key_str_to_id_map = key_str_to_id_map.read().unwrap();
            let Some(cert_id) = key_str_to_id_map.get(&key_str) else {
                return Ok(None);
            };
            let inner = inner.read().unwrap();
            let Some(cert) = inner.get_cert(cert_id) else {
                return Ok(None);
            };
            Ok(Some(cert.bytes()))
        })
        .await
    }

    async fn write(
        &self,
        key_str: String,
        cert_type: CertType,
        contents: impl AsRef<[u8]>,
    ) -> Result<(), std::io::Error> {
        let key_str_to_id_map = self.key_str_to_id_map.clone();
        let inner = self.inner.clone();
        let bytes = contents.as_ref().to_vec();
        unblock(move || {
            let mut bytes = Some(bytes);
            if let Some(cert_id) = key_str_to_id_map.read().unwrap().get(&key_str) {
                // cert already exists somehow ...
                let mut inner = inner.write().unwrap();
                inner.get_cert_mut(cert_id, |cert| {
                    cert.set_bytes(bytes.take().unwrap());
                });
                Ok(())
            } else {
                // cert is new
                let mut inner = inner.write().unwrap();
                let new_cert = Cert::new(key_str.clone(), cert_type, bytes.take().unwrap());
                let cert_id = inner
                    .create_cert(new_cert)
                    .map_err(|e| std::io::Error::new(ErrorKind::InvalidData, e))?;

                let mut key_str_to_id_map = key_str_to_id_map.write().unwrap();
                key_str_to_id_map.insert(key_str, cert_id);

                Ok(())
            }
        })
        .await
    }

    fn cached_account_file_name(contact: &[String], directory_url: impl AsRef<str>) -> String {
        let mut ctx = Context::new(&SHA256);
        for el in contact {
            ctx.update(el.as_ref());
            ctx.update(&[0])
        }
        ctx.update(directory_url.as_ref().as_bytes());
        let hash = BASE64_URL_SAFE_NO_PAD.encode(ctx.finish());
        format!("cached_account_{}", hash)
    }

    fn cached_cert_file_name(domains: &[String], directory_url: impl AsRef<str>) -> String {
        let mut ctx = Context::new(&SHA256);
        for domain in domains {
            ctx.update(domain.as_ref());
            ctx.update(&[0])
        }
        ctx.update(directory_url.as_ref().as_bytes());
        let hash = BASE64_URL_SAFE_NO_PAD.encode(ctx.finish());
        format!("cached_cert_{}", hash)
    }
}

#[async_trait]
impl CertCache for GitCache {
    type EC = std::io::Error;
    async fn load_cert(
        &self,
        domains: &[String],
        directory_url: &str,
    ) -> Result<Option<Vec<u8>>, Self::EC> {
        let file_name = Self::cached_cert_file_name(&domains, directory_url);
        self.read_if_exist(file_name).await
    }
    async fn store_cert(
        &self,
        domains: &[String],
        directory_url: &str,
        cert: &[u8],
    ) -> Result<(), Self::EC> {
        let file_name = Self::cached_cert_file_name(&domains, directory_url);
        self.write(file_name, CertType::Cert, cert).await
    }
}

#[async_trait]
impl AccountCache for GitCache {
    type EA = std::io::Error;
    async fn load_account(
        &self,
        contact: &[String],
        directory_url: &str,
    ) -> Result<Option<Vec<u8>>, Self::EA> {
        let file_name = Self::cached_account_file_name(&contact, directory_url);
        self.read_if_exist(file_name).await
    }

    async fn store_account(
        &self,
        contact: &[String],
        directory_url: &str,
        account: &[u8],
    ) -> Result<(), Self::EA> {
        let file_name = Self::cached_account_file_name(&contact, directory_url);
        self.write(file_name, CertType::Account, account).await
    }
}
