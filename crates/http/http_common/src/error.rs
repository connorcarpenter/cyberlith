use crate::Response;

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

    pub fn to_response(&self, url: &str) -> Response {
        let mut response = Response::default();
        response.url = url.to_string();
        response.ok = false;
        response.status = match self {
            ResponseError::Unauthenticated => 401,
            ResponseError::NotFound => 404,
            ResponseError::InternalServerError(_) => 500,
            ResponseError::NetworkError(_) => 500,
            ResponseError::SerdeError => 500,
        };
        response.status_text = self.to_string();
        response
    }

    pub fn from_response(response: &Response) -> ResponseError {
        match response.status {
            200 => panic!("not an error"),
            401 => ResponseError::Unauthenticated,
            404 => ResponseError::NotFound,
            500 => ResponseError::InternalServerError(response.status_text.clone()),
            status_code => panic!("unexpected status code: {}", status_code),
        }
    }
}
