
pub enum ResponseError {
    None,
    HttpError(String),
    SerdeError,
    Unauthenticated,
    ChannelRecvError,
    InternalServerError(String),
    NotFound,
}

impl ResponseError {
    pub fn to_string(&self) -> String {
        match self {
            ResponseError::None => "None".to_string(),
            ResponseError::HttpError(error) => format!("HttpError: {}", error),
            ResponseError::SerdeError => "SerdeError".to_string(),
            ResponseError::Unauthenticated => "Unauthenticated".to_string(),
            ResponseError::ChannelRecvError => "ChannelRecvError".to_string(),
            ResponseError::InternalServerError(error) => format!("InternalServerError: {}", error),
            ResponseError::NotFound => "NotFound".to_string(),
        }
    }
}