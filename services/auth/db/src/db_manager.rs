use auth_server_types::UserId;
use db::{DatabaseWrapper, DbError};

use crate::{
    error::AuthServerDbError,
    user::{User, DbUserId, Users},
};

pub struct DatabaseManager {
    wrapper: DatabaseWrapper,
}

impl DatabaseManager {
    pub fn init() -> Self {
        let mut wrapper = DatabaseWrapper::init();
        wrapper.table_open::<Users>();
        Self { wrapper }
    }

    // user create
    pub fn create_user(&mut self, user: User) -> Result<UserId, AuthServerDbError> {
        self.wrapper
            .table_mut::<Users>()
            .insert(user)
            .map(|id| id.into())
            .map_err(|err| match err {
                DbError::KeyAlreadyExists => AuthServerDbError::InsertedDuplicateUserId,
            })
    }

    // user read
    pub fn get_user(&self, id: &UserId) -> Option<&User> {
        let id: u64 = (*id).into();
        let id = DbUserId::from(id);
        self.wrapper.table::<Users>().get(&id)
    }

    // user update
    pub fn get_user_mut<F: FnMut(&mut User)>(&mut self, id: &UserId, func: F) {
        let id = DbUserId::from(*id);
        self.wrapper.table_mut::<Users>().get_mut(&id, func);
    }

    // user delete
    pub fn delete_user(&mut self, id: &UserId) {
        let id = DbUserId::from(*id);
        self.wrapper.table_mut::<Users>().remove(&id);
    }

    // user list
    pub fn list_users(&self) -> Vec<(UserId, &User)> {
        self.wrapper.table::<Users>().list().into_iter().map(|(k, v)| ((*k).into(), v)).collect()
    }
}
