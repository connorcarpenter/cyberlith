
use auth_server_db::DatabaseManager;

pub struct State {
    database_manager: DatabaseManager,
}

impl State {
    pub fn new() -> Self {
        Self {
            database_manager: DatabaseManager::init(),
        }
    }
}
