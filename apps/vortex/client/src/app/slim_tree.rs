use vortex_proto::components::EntryKind;

pub struct SlimTree {
    pub name: String,
    pub kind: EntryKind,
    pub children: Option<Vec<SlimTree>>,
}

impl SlimTree {
    pub fn new(name: String, kind: EntryKind) -> Self {
        Self {
            name,
            kind,
            children: None,
        }
    }
}