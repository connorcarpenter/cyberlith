use auth_server_http_proto::{AccessToken, RefreshToken};

// this is session-temporary data about each user, should be mostly just tokens?
pub struct UserData {
    pub(crate) user_name: String, // and name...
    pub(crate) current_access_token: Option<AccessToken>,
    pub(crate) current_refresh_token: Option<RefreshToken>,
}

impl UserData {
    pub(crate) fn new(user_name: String) -> Self {
        Self {
            user_name,
            current_access_token: None,
            current_refresh_token: None,
        }
    }
}
