use auth_server_http_proto::UserRegisterRequest;

pub struct TempRegistration {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl From<UserRegisterRequest> for TempRegistration {
    fn from(req: UserRegisterRequest) -> Self {
        Self {
            name: req.username,
            email: req.email,
            password: req.password,
        }
    }
}
