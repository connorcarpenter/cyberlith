use std::{fmt::{Debug, Display, Formatter}, error::Error};

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
