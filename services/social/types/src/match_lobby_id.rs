use naia_serde::SerdeInternal as Serde;

#[derive(Serde, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
pub struct MatchLobbyId {
    val: u64,
}

impl MatchLobbyId {
    pub fn new(val: u64) -> Self {
        Self { val }
    }
}

impl From<u64> for MatchLobbyId {
    fn from(val: u64) -> Self {
        Self { val }
    }
}

impl Into<u64> for MatchLobbyId {
    fn into(self) -> u64 {
        self.val
    }
}