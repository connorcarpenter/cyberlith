
pub enum ResponseError {
    InvalidResponse,
    IoError(String),
}

impl ResponseError {
    pub fn to_string(&self) -> String {
        match self {
            ResponseError::InvalidResponse => "InvalidResponse".to_string(),
            ResponseError::IoError(e) => format!("IoError: {}", e),
        }
    }
}
