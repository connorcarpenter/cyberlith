pub(crate) struct EmailCatalog {
    register_verification_txt: String,
    register_verification_html: String,
    user_name_forgot_txt: String,
    user_name_forgot_html: String,
    user_password_forgot_txt: String,
    user_password_forgot_html: String,
}

impl EmailCatalog {
    pub fn new() -> Self {
        Self {
            register_verification_txt: include_str!("register_email_verification.txt").to_string(),
            register_verification_html: include_str!("register_email_verification.html")
                .to_string(),
            user_name_forgot_txt: include_str!("user_name_forgot.txt").to_string(),
            user_name_forgot_html: include_str!("user_name_forgot.html").to_string(),
            user_password_forgot_txt: include_str!("user_password_forgot.txt").to_string(),
            user_password_forgot_html: include_str!("user_password_forgot.html").to_string(),
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

    pub fn user_name_forgot_txt(&self, username: &str, link_url: &str) -> String {
        self.user_name_forgot_txt
            .replace("{username}", username)
            .replace("{link_url}", link_url)
    }

    pub fn user_name_forgot_html(&self, username: &str, link_url: &str) -> String {
        self.user_name_forgot_html
            .replace("{username}", username)
            .replace("{link_url}", link_url)
    }

    pub fn user_password_forgot_txt(&self, username: &str, link_url: &str) -> String {
        self.user_password_forgot_txt
            .replace("{username}", username)
            .replace("{link_url}", link_url)
    }

    pub fn user_password_forgot_html(&self, username: &str, link_url: &str) -> String {
        self.user_password_forgot_html
            .replace("{username}", username)
            .replace("{link_url}", link_url)
    }
}
