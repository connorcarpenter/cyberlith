use std::fmt::Debug;

use naia_bevy_shared::sequence_greater_than;

use naia_serde::{SerdeInternal as Serde, UnsignedInteger};

#[derive(Serde, PartialEq, Clone, Eq, Copy, Hash, Debug)]
pub struct MatchLobbyId {
    id: UnsignedInteger<14>, // TODO: I can't imagine more than 10000 lobbies at a time ... not a bad cap
}

impl MatchLobbyId {
    pub fn new(id: u16) -> Self {
        Self {
            id: UnsignedInteger::new(id),
        }
    }

    pub fn next(&self) -> Self {
        let id: u16 = self.id.to();
        let mut id = id + 1;
        let max_value: u16 = 2_u16.pow(14);
        if id == max_value {
            id = 0;
        }
        Self::new(id)
    }
}

impl PartialOrd for MatchLobbyId {
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

impl Ord for MatchLobbyId {
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
