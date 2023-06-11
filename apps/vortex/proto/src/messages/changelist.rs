use naia_bevy_shared::{EntityProperty, Message, Serde};

#[derive(Serde, PartialEq, Clone)]
pub enum ChangelistAction {
    CommitAll,
    CommitSingle,
    Rollback,
}

#[derive(Message)]
pub struct ChangelistMessage {
    pub entity: EntityProperty,
    pub action: ChangelistAction,
}

impl ChangelistMessage {
    pub fn new(action: ChangelistAction) -> Self {
        Self {
            entity: EntityProperty::new(),
            action,
        }
    }
}
