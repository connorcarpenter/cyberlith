use game_engine::config::{PUBLIC_IP_ADDR, SUBDOMAIN_WWW, SUBDOMAIN_API};

pub(crate) fn get_www_url() -> String {
    if SUBDOMAIN_WWW.is_empty() {
        PUBLIC_IP_ADDR.to_string()
    } else {
        format!("{}.{}", SUBDOMAIN_WWW, PUBLIC_IP_ADDR)
    }
}

pub(crate) fn get_api_url() -> String {
    if SUBDOMAIN_API.is_empty() {
        PUBLIC_IP_ADDR.to_string()
    } else {
        format!("{}.{}", SUBDOMAIN_API, PUBLIC_IP_ADDR)
    }
}