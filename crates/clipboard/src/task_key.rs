
// TaskKey
pub struct TaskKey {
    pub(crate) id: u64,
}

impl TaskKey {
    pub fn new(id: u64) -> Self {
        Self {
            id,
        }
    }
}

impl Clone for TaskKey {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
        }
    }
}

impl Copy for TaskKey {}

impl PartialEq<Self> for TaskKey {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TaskKey {}
