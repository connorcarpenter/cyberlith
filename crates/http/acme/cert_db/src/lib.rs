// this layer should be specific to the a server's ACME cert/account storage needs
// it should not have any other business logic here
// just details on how to CRUD the underlying DB

mod db_manager;
pub use db_manager::DatabaseManager;

mod cert;
pub use cert::{Cert, CertId, CertType};

mod error;
pub use error::CertDbError;
