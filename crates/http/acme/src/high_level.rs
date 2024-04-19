use std::io::Error;

use rustls_acme::AcmeConfig;

use crate::git_cache::GitCache;

pub struct Config {
    /// Domains
    domains: Vec<String>,

    /// Contact info
    email: Vec<String>,

    /// Use Let's Encrypt production environment
    /// (see https://letsencrypt.org/docs/staging-environment/)
    prod: bool,
}

impl Config {
    pub fn new(prod: bool, domains: Vec<String>, email: Vec<String>) -> Self {
        Self {
            domains,
            email,
            prod,
        }
    }

    pub fn to_acme_config(self) -> AcmeConfig<Error, Error> {
        AcmeConfig::new(self.domains)
            .contact(self.email.iter().map(|e| format!("mailto:{}", e)))
            .cache_option(Some(GitCache::new()))
            .directory_lets_encrypt(self.prod)
    }
}
