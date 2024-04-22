use naia_bevy_shared::{Message, Serde};

#[derive(Serde, PartialEq, Clone)]
pub struct AuthInner {
    pub token: String,
}

impl AuthInner {
    pub fn new(token: &str) -> Self {
        Self {
            token: token.to_string(),
        }
    }

    pub fn to_outer(&self) -> Auth {
        Auth {
            inner: self.clone(),
        }
    }
}

#[derive(Message)]
pub struct Auth {
    inner: AuthInner,
}

impl Auth {
    pub fn new(token: &str) -> Self {
        Self {
            inner: AuthInner::new(token),
        }
    }

    pub fn token(&self) -> &str {
        &self.inner.token
    }
}
