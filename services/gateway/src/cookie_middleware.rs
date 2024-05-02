
use config::{PUBLIC_IP_ADDR, TargetEnv};
use http_server::Response;

use auth_server_http_proto::{AccessToken, RefreshToken};

pub(crate) fn response_set_cookies_tokens(
    response: &mut Response,
    access_token: &AccessToken,
    refresh_token: &RefreshToken
) {
    // access token
    {
        const ONE_DAY_IN_SECONDS: u32 = 60 * 60 * 24;
        let access_token_value = get_set_cookie_value("access_token", &access_token.to_string(), ONE_DAY_IN_SECONDS);
        response.insert_header(
            "Set-Cookie",
            &access_token_value,
        );
    }

    // refresh token
    {
        const ONE_WEEK_IN_SECONDS: u32 = 60 * 60 * 24 * 7;
        let refresh_token_value = get_set_cookie_value("refresh_token", &refresh_token.to_string(), ONE_WEEK_IN_SECONDS);
        response.insert_header(
            "Set-Cookie",
            &refresh_token_value,
        );
    }
}

pub(crate) fn get_set_cookie_value(
    cookie_name: &str,
    cookie_value: &str,
    max_age_secs: u32,
) -> String {
    let cookie_attributes = match TargetEnv::get() {
        TargetEnv::Local => format!("; HttpOnly; SameSite=Lax; Path=/; Domain={}; Max-Age={}", PUBLIC_IP_ADDR, max_age_secs),
        TargetEnv::Prod => format!("; Secure; HttpOnly; SameSite=Lax; Path=/; Domain={}; Max-Age={}", PUBLIC_IP_ADDR, max_age_secs),
    };
    format!(
        "{}={}{}",
        cookie_name,
        cookie_value,
        cookie_attributes,
    )
}