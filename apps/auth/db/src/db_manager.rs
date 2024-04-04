use db::{DatabaseWrapper};

use crate::{user::{Users, User, UserId}};

pub struct DatabaseManager {
    wrapper: DatabaseWrapper,
}

impl DatabaseManager {

    pub fn init() -> Self {
        let mut wrapper = DatabaseWrapper::init();
        wrapper.table_open::<Users>("cyberlith_users");
        Self {
            wrapper,
        }
    }

    // user create
    pub fn create_user(&mut self, user: User) -> UserId {
        self.wrapper.table_mut::<Users>().insert(user)
    }

    // user read
    pub fn get_user(&self, id: &UserId) -> Option<&User> {
        self.wrapper.table::<Users>().get(id)
    }

    // user update
    pub fn get_user_mut(&mut self, id: &UserId) -> Option<&mut User> {
        self.wrapper.table_mut::<Users>().get_mut(id)
    }

    // user delete
    pub fn delete_user(&mut self, id: &UserId) {
        self.wrapper.table_mut::<Users>().remove(id);
    }
}