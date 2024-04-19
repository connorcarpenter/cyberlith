use db::{DatabaseWrapper, DbError};

use crate::{
    cert::{Cert, CertId, Certs},
    error::CertDbError,
};

pub struct DatabaseManager {
    wrapper: DatabaseWrapper,
}

impl DatabaseManager {
    pub fn init() -> Self {
        let mut wrapper = DatabaseWrapper::init();
        wrapper.table_open::<Certs>();
        Self { wrapper }
    }

    // cert create
    pub fn create_cert(&mut self, cert: Cert) -> Result<CertId, CertDbError> {
        self.wrapper
            .table_mut::<Certs>()
            .insert(cert)
            .map_err(|err| match err {
                DbError::KeyAlreadyExists => CertDbError::InsertedDuplicateCertId,
            })
    }

    // cert read
    pub fn get_cert(&self, id: &CertId) -> Option<&Cert> {
        self.wrapper.table::<Certs>().get(id)
    }

    // cert update
    pub fn get_cert_mut<F: FnMut(&mut Cert)>(&mut self, id: &CertId, func: F) {
        self.wrapper.table_mut::<Certs>().get_mut(id, func);
    }

    // cert delete
    pub fn delete_cert(&mut self, id: &CertId) {
        self.wrapper.table_mut::<Certs>().remove(id);
    }

    // cert list
    pub fn list_certs(&self) -> Vec<(&CertId, &Cert)> {
        self.wrapper.table::<Certs>().list()
    }
}
