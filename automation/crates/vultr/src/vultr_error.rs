use std::fmt::Display;

#[derive(Debug)]
pub enum VultrError {
    Dashboard(String),
    Reqwest(reqwest::Error),
    VultrApi(String),
    Json(),
}

impl From<reqwest::Error> for VultrError {
    fn from(value: reqwest::Error) -> Self {
        VultrError::Reqwest(value)
    }
}

impl Display for VultrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            VultrError::Dashboard(err) => write!(f, "Dashboard: {}", err),
            VultrError::Reqwest(err) => write!(f, "Request: {}", err),
            VultrError::VultrApi(err) => write!(f, "VultrApi: {}", err),
            VultrError::Json() => write!(f, "JSON"),
        }
    }
}

impl std::error::Error for VultrError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}
