
use auth_server_http_proto::UserLoginResponse;

use http_client::ResponseError;
use http_server::{ApiResponse, Response};

use crate::target_env::{get_env, TargetEnv};

pub(crate) async fn response_set_cookie(
    mut response: Response,
) -> Result<Response, ResponseError> {

    let Ok(user_login_response) = UserLoginResponse::from_response(response.clone()) else {
        return Err(ResponseError::SerdeError);
    };

    // put access token into user cookie

    let cookie_attributes = match get_env() {
        TargetEnv::Local => "".to_string(),
        TargetEnv::Prod => {
            let mut expire_time_utc = chrono::Utc::now();
            let expire_duration_1_week = chrono::Duration::weeks(1);
            expire_time_utc += expire_duration_1_week;

            format!(
                "; Secure; HttpOnly; SameSite=Lax; Domain=.cyberlith.com; Expires={}",
                expire_time_utc
            )
        },
    };

    let set_cookie_value = format!(
        "access_token={}{}",
        user_login_response.access_token,
        cookie_attributes,
    );
    response.set_header(
        "Set-Cookie",
        &set_cookie_value,
    );

    Ok(response)
}