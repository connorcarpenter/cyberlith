use naia_bevy_shared::{EntityProperty, Message, Serde};

#[derive(Serde, PartialEq, Clone, Debug, Copy)]
pub enum ChangelistAction {
    Commit,
    Rollback,
}

#[derive(Message)]
pub struct ChangelistMessage {
    pub entity: EntityProperty,
    pub action: ChangelistAction,
    pub commit_message: Option<String>,
}

impl ChangelistMessage {
    pub fn new(action: ChangelistAction, commit_message: Option<&str>) -> Self {
        Self {
            entity: EntityProperty::new(),
            action,
            commit_message: commit_message.map(|s| s.to_string()),
        }
    }
}
