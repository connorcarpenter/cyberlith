
pub enum FsTaskError {
    InvalidResult,
    IoError(String),
}

impl FsTaskError {
    pub fn to_string(&self) -> String {
        match self {
            FsTaskError::InvalidResult => "InvalidResult".to_string(),
            FsTaskError::IoError(e) => format!("IoError: {}", e),
        }
    }
}
