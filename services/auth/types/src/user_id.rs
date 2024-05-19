use naia_serde::SerdeInternal as Serde;

#[derive(Serde, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Debug)]
pub struct UserId {
    val: u64,
}

impl UserId {
    pub fn new(val: u64) -> Self {
        Self { val }
    }
}

impl From<u64> for UserId {
    fn from(val: u64) -> Self {
        Self { val }
    }
}

impl Into<u64> for UserId {
    fn into(self) -> u64 {
        self.val
    }
}
