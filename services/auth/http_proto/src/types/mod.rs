mod access_token;
mod refresh_token;
mod register_token;
mod reset_password_token;

pub use access_token::AccessToken;
pub use refresh_token::RefreshToken;
pub use register_token::RegisterToken;
pub use reset_password_token::ResetPasswordToken;

pub(crate) fn get_set_cookie_value(
    cookie_name: &str,
    cookie_value: &str,
    domain: &str,
    max_age_secs: u32,
    secure: bool,
) -> String {
    let secure = if secure { "; Secure" } else { "" };
    format!(
        "{}={}{}; HttpOnly; SameSite=Lax; Path=/; Domain={}; Max-Age={}",
        cookie_name, cookie_value, secure, domain, max_age_secs
    )
}
