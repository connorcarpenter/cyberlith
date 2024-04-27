pub enum RequestError {
    None,
    SerdeError,
}

// keep in mind these errors are probably not returned from the Gateway server, just internal servers
pub enum ResponseError {
    // client errors, from client
    NetworkError(String),
    SerdeError,

    // client errors, from server
    Unauthenticated, // 401 response
    NotFound, // 404 response

    // server errors, from server
    InternalServerError(String), // 500 response
}

impl ResponseError {
    pub fn to_string(&self) -> String {
        match self {
            ResponseError::NetworkError(error) => format!("NetworkError: {}", error),
            ResponseError::SerdeError => "SerdeError".to_string(),
            ResponseError::Unauthenticated => "Unauthenticated".to_string(),
            ResponseError::InternalServerError(error) => format!("InternalServerError: {}", error),
            ResponseError::NotFound => "NotFound".to_string(),
        }
    }
}
