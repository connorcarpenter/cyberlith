use std::fmt::Display;

use vultr::VultrError;

#[derive(Debug)]
pub enum CliError {
    Message(String),
    Vultr(VultrError),
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            CliError::Message(err) => write!(f, "Error: {}", err),
            CliError::Vultr(err) => write!(f, "Vultr Error: {:?}", err),
        }
    }
}

impl std::error::Error for CliError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }

    fn cause(&self) -> Option<&dyn std::error::Error> {
        self.source()
    }
}

impl From<VultrError> for CliError {
    fn from(err: VultrError) -> Self {
        Self::Vultr(err)
    }
}