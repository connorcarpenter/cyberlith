use chrono::{DateTime, Utc};

use auth_server_http_proto::{RefreshTokenGrantResponse, UserLoginResponse};
use http_client::ResponseError;
use http_server::{ApiResponse, Response};

use crate::target_env::{get_env, TargetEnv};

pub(crate) trait SetCookieResponse: ApiResponse {
    fn access_token(&self) -> &str;
}

impl SetCookieResponse for UserLoginResponse {
    fn access_token(&self) -> &str {
        &self.access_token
    }
}

impl SetCookieResponse for RefreshTokenGrantResponse {
    fn access_token(&self) -> &str {
        &self.access_token
    }
}

pub(crate) async fn handler<R: SetCookieResponse>(
    response: Response,
) -> Result<Response, ResponseError> {

    match response.to_result() {
        Ok(mut response) => {
            let Ok(typed_response) = R::from_response(response.clone()) else {
                return Err(ResponseError::SerdeError);
            };

            // put access token into user cookie
            let env = get_env();
            let access_token = typed_response.access_token();

            let expire_time_utc_opt = match env {
                TargetEnv::Local => None,
                TargetEnv::Prod => {
                    let mut expire_time_utc = chrono::Utc::now();
                    let expire_duration_1_week = chrono::Duration::weeks(1);
                    expire_time_utc += expire_duration_1_week;
                    Some(expire_time_utc)
                }
            };

            let set_cookie_value = get_set_cookie_value(env, access_token, expire_time_utc_opt);
            response.set_header(
                "Set-Cookie",
                &set_cookie_value,
            );

            Ok(response)
        }
        Err(e) => {
            Err(e)
        }
    }
}

pub(crate) fn get_set_cookie_value(
    target_env: TargetEnv,
    access_token: &str,
    expire_time_utc_opt: Option<DateTime<Utc>>
) -> String {
    let cookie_attributes = match target_env {
        TargetEnv::Local => "".to_string(),
        TargetEnv::Prod => "; Secure; HttpOnly; SameSite=Lax; Domain=.cyberlith.com".to_string(),
    };
    let expire_str = match expire_time_utc_opt {
        Some(expire_time_utc) => format!("; Expires={}", expire_time_utc),
        None => "".to_string(),
    };
    format!(
        "access_token={}{}{}",
        access_token,
        cookie_attributes,
        expire_str,
    )
}