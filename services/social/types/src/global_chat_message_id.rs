use std::fmt::Debug;

use naia_bevy_shared::sequence_greater_than;

use naia_serde::{SerdeInternal as Serde};

#[derive(Serde, PartialEq, Clone, Eq, Copy, Hash, Debug)]
pub struct GlobalChatMessageId {
    id: u16,
}

impl GlobalChatMessageId {
    pub fn new(id: u16) -> Self {
        Self { id }
    }

    pub fn next(&self) -> Self {
        let val = self.id.wrapping_add(1);
        Self::new(val)
    }
}

impl PartialOrd for GlobalChatMessageId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.id == other.id {
            return Some(std::cmp::Ordering::Equal);
        }

        if sequence_greater_than(self.id, other.id) {
            return Some(std::cmp::Ordering::Greater);
        } else {
            return Some(std::cmp::Ordering::Less);
        }
    }
}

impl Ord for GlobalChatMessageId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.id == other.id {
            return std::cmp::Ordering::Equal;
        }

        if sequence_greater_than(self.id, other.id) {
            return std::cmp::Ordering::Greater;
        } else {
            return std::cmp::Ordering::Less;
        }
    }
}