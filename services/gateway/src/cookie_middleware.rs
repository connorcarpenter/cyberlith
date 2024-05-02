
use config::{PUBLIC_IP_ADDR, TargetEnv};
use http_server::{ApiResponse, Response};

use auth_server_http_proto::{AccessToken, RefreshToken};

pub(crate) trait SetCookieResponse: ApiResponse {
    fn access_token(&self) -> &AccessToken;
}

// pub(crate) async fn set_cookie_on_200<R: SetCookieResponse>(
//     response: Response,
// ) -> Result<Response, ResponseError> {
//
//     match response.to_result() {
//         Ok(mut response) => {
//             let Ok(typed_response) = R::from_response(response.clone()) else {
//                 return Err(ResponseError::SerdeError);
//             };
//
//             // put access token into user cookie
//             let env = TargetEnv::get();
//             let access_token = typed_response.access_token();
//
//             let expire_time_utc_opt = match env {
//                 TargetEnv::Local => None,
//                 TargetEnv::Prod => {
//                     let mut expire_time_utc = chrono::Utc::now();
//                     let expire_duration_1_week = chrono::Duration::weeks(1);
//                     expire_time_utc += expire_duration_1_week;
//                     Some(expire_time_utc)
//                 }
//             };
//
//             let set_cookie_value = get_set_cookie_value("access_token", &access_token.to_string(), expire_time_utc_opt);
//             response.insert_header(
//                 "Set-Cookie",
//                 &set_cookie_value,
//             );
//
//             Ok(response)
//         }
//         Err(e) => {
//             Err(e)
//         }
//     }
// }

// pub(crate) async fn handler_clear_cookie_on_401(
//     mut response: Response,
// ) -> Result<Response, ResponseError> {
//     if response.status == 401 {
//         clear_cookie(&mut response);
//     }
//
//     Ok(response)
// }

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

pub(crate) fn response_clear_access_token(response: &mut Response) {
    let cookie_val = get_set_cookie_value("access_token", "", 0);
    response.insert_header("Set-Cookie", &cookie_val);
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