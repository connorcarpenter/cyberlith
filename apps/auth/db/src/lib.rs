// this layer should be specific to the AuthServer's DB needs
// it should not have any other business logic here
// just details on how to CRUD the underlying DB

mod db_manager;
pub use db_manager::DatabaseManager;

mod user;
pub use user::User;
