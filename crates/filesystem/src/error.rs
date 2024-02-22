
pub enum TaskError {
    InvalidResult,
    IoError(String),
}

impl TaskError {
    pub fn to_string(&self) -> String {
        match self {
            TaskError::InvalidResult => "InvalidResult".to_string(),
            TaskError::IoError(e) => format!("IoError: {}", e),
        }
    }
}
