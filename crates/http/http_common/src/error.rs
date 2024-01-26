pub enum RequestError {
    None,
    SerdeError,
}

pub enum ResponseError {
    None,
    HttpError(String),
    SerdeError,
    Unauthorized,
    ChannelRecvError,
    InternalServerError(String),
}

impl ResponseError {
    pub fn to_string(&self) -> String {
        match self {
            ResponseError::None => "None".to_string(),
            ResponseError::HttpError(error) => format!("HttpError: {}", error),
            ResponseError::SerdeError => "SerdeError".to_string(),
            ResponseError::Unauthorized => "Unauthorized".to_string(),
            ResponseError::ChannelRecvError => "ChannelRecvError".to_string(),
            ResponseError::InternalServerError(error) => format!("InternalServerError: {}", error),
        }
    }
}