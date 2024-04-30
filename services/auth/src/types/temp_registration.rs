
use logging::warn;

use auth_server_http_proto::UserRegisterRequest;

use crate::error::AuthServerError;

pub struct TempRegistration {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl TempRegistration {
    pub(crate) fn from_req(req: UserRegisterRequest) -> Result<Self, AuthServerError> {
        let password_hash = crypto::password_hasher::process(&req.password).map_err(|e| {
            warn!("password_hasher::hash failed: {:?}", e);
            AuthServerError::PasswordHashError
        })?;
        Ok(Self {
            name: req.username,
            email: req.email,
            password: password_hash,
        })
    }
}
