use std::{
    error::Error,
    fmt::{Debug, Display, Formatter},
};

#[derive(Debug)]
pub enum CertDbError {
    InsertedDuplicateCertId,
}

impl Display for CertDbError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CertDbError::InsertedDuplicateCertId => write!(f, "InsertedDuplicateCertId"),
        }
    }
}

impl Error for CertDbError {}
