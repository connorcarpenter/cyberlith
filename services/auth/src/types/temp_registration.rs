
use logging::warn;

use crate::error::AuthServerError;

pub struct TempRegistration {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl TempRegistration {
    pub(crate) fn new(name: &str, email: &str, password: &str) -> Result<Self, AuthServerError> {
        let password_hash = crypto::password_hasher::process(password).map_err(|e| {
            warn!("password_hasher::hash failed: {:?}", e);
            AuthServerError::PasswordHashError
        })?;
        Ok(Self {
            name: name.to_string(),
            email: email.to_string(),
            password: password_hash,
        })
    }
}
