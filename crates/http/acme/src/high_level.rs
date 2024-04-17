use std::{io::Error, path::PathBuf};

use crate::{caches::DirCache, config::AcmeConfig};

pub struct Config {
    /// Domains
    domains: Vec<String>,

    /// Contact info
    email: Vec<String>,

    /// Cache directory
    cache: Option<PathBuf>,

    /// Use Let's Encrypt production environment
    /// (see https://letsencrypt.org/docs/staging-environment/)
    prod: bool,
}

impl Config {
    pub fn new(
        prod: bool,
        domains: Vec<String>,
        email: Vec<String>,
        cache: Option<PathBuf>,
    ) -> Self {
        Self {
            domains,
            email,
            cache,
            prod,
        }
    }

    pub fn to_acme_config(self) -> AcmeConfig<Error, Error> {
        AcmeConfig::new(self.domains)
            .contact(self.email.iter().map(|e| format!("mailto:{}", e)))
            .cache_option(self.cache.clone().map(DirCache::new))
            .directory_lets_encrypt(self.prod)
    }
}
