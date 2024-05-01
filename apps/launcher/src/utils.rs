use game_engine::config::{PUBLIC_IP_ADDR, SUBDOMAIN_WWW};

pub(crate) fn get_www_url() -> String {
    if SUBDOMAIN_WWW.is_empty() {
        PUBLIC_IP_ADDR.to_string()
    } else {
        format!("{}.{}", SUBDOMAIN_WWW, PUBLIC_IP_ADDR)
    }
}