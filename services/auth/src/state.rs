use std::collections::HashMap;

use auth_server_db::{DatabaseManager, UserId};

use crate::{types::{RegisterToken, TempRegistration}, emails::EmailCatalog};

pub struct State {
    pub(crate) database_manager: DatabaseManager,
    pub(crate) email_catalog: EmailCatalog,
    pub(crate) temp_regs: HashMap<RegisterToken, TempRegistration>,
    pub(crate) username_to_id_map: HashMap<String, Option<UserId>>,
    pub(crate) email_to_id_map: HashMap<String, Option<UserId>>,
}

impl State {
    pub fn new() -> Self {

        let database_manager = DatabaseManager::init();
        let mut username_to_id_map = HashMap::new();
        let mut email_to_id_map = HashMap::new();

        for (id, user) in database_manager.list_users() {
            username_to_id_map.insert(user.username().to_string(), Some(*id));
            email_to_id_map.insert(user.email().to_string(), Some(*id));
        }

        Self {
            email_catalog: EmailCatalog::new(),
            database_manager,
            temp_regs: HashMap::new(),
            username_to_id_map,
            email_to_id_map,
        }
    }
}

