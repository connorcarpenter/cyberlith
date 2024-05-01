use chrono::{DateTime, TimeZone, Utc};

use auth_server_http_proto::{AccessToken, RefreshTokenGrantResponse, UserLoginResponse};
use config::TargetEnv;
use http_client::ResponseError;
use http_server::{ApiResponse, Response};

pub(crate) trait SetCookieResponse: ApiResponse {
    fn access_token(&self) -> &AccessToken;
}

impl SetCookieResponse for UserLoginResponse {
    fn access_token(&self) -> &AccessToken {
        &self.access_token
    }
}

impl SetCookieResponse for RefreshTokenGrantResponse {
    fn access_token(&self) -> &AccessToken {
        &self.access_token
    }
}

pub(crate) async fn set_cookie_on_200<R: SetCookieResponse>(
    response: Response,
) -> Result<Response, ResponseError> {

    match response.to_result() {
        Ok(mut response) => {
            let Ok(typed_response) = R::from_response(response.clone()) else {
                return Err(ResponseError::SerdeError);
            };

            // put access token into user cookie
            let env = TargetEnv::get();
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

            let set_cookie_value = get_set_cookie_value("access_token", &access_token.to_string(), expire_time_utc_opt);
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

pub(crate) async fn handler_clear_cookie_on_401(
    mut response: Response,
) -> Result<Response, ResponseError> {
    if response.status == 401 {
        clear_cookie(&mut response);
    }

    Ok(response)
}

pub(crate) fn clear_cookie(response: &mut Response) {
    let earliest_utc_time = chrono::Utc.timestamp_nanos(0);
    let cookie_val = get_set_cookie_value("access_token", "", Some(earliest_utc_time));
    response.set_header("Set-Cookie", &cookie_val);
}

pub(crate) fn get_set_cookie_value(
    cookie_name: &str,
    cookie_value: &str,
    expire_time_utc_opt: Option<DateTime<Utc>>
) -> String {
    let cookie_attributes = match TargetEnv::get() {
        TargetEnv::Local => "".to_string(),
        TargetEnv::Prod => "; Secure; HttpOnly; SameSite=Lax; Domain=.cyberlith.com".to_string(),
    };
    let expire_str = match expire_time_utc_opt {
        Some(expire_time_utc) => format!("; Expires={}", expire_time_utc),
        None => "".to_string(),
    };
    format!(
        "{}={}{}{}",
        cookie_name,
        cookie_value,
        cookie_attributes,
        expire_str,
    )
}