pub(crate) struct EmailCatalog {
    register_verification_txt: String,
    register_verification_html: String,
}

impl EmailCatalog {
    pub fn new() -> Self {
        Self {
            register_verification_txt: include_str!("register_email_verification.txt").to_string(),
            register_verification_html: include_str!("register_email_verification.html").to_string(),
        }
    }

    pub fn register_verification_txt(&self, username: &str, link_url: &str) -> String {
        self.register_verification_txt
            .replace("{username}", username)
            .replace("{link_url}", link_url)
    }

    pub fn register_verification_html(&self, username: &str, link_url: &str) -> String {
        self.register_verification_html
            .replace("{username}", username)
            .replace("{link_url}", link_url)
    }
}