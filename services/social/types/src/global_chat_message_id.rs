use std::fmt::Debug;

use naia_bevy_shared::sequence_greater_than;

use naia_serde::{SerdeInternal as Serde, UnsignedInteger};

#[derive(Serde, PartialEq, Clone, Eq, Copy, Hash, Debug)]
pub struct GlobalChatMessageId {
    id: UnsignedInteger<9>, // TODO: there should only ever be ... 100 message in chat at a time? in that case, u8 would be enough
}

impl GlobalChatMessageId {
    pub fn new(id: u16) -> Self {
        Self {
            id: UnsignedInteger::new(id),
        }
    }

    pub fn next(&self) -> Self {
        let id: u16 = self.id.to();
        let mut id = id + 1;
        let max_value: u16 = 2_u16.pow(9);
        if id == max_value {
            id = 0;
        }
        Self::new(id)
    }
}

impl PartialOrd for GlobalChatMessageId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.id == other.id {
            return Some(std::cmp::Ordering::Equal);
        }

        let a: u16 = self.id.to();
        let b: u16 = other.id.to();
        if sequence_greater_than(a, b) {
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

        let a: u16 = self.id.to();
        let b: u16 = other.id.to();
        if sequence_greater_than(a, b) {
            return std::cmp::Ordering::Greater;
        } else {
            return std::cmp::Ordering::Less;
        }
    }
}