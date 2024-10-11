use naia_bevy_shared::Message;

#[derive(Message)]
pub struct KeyCommand {
    pub w: bool,
    pub s: bool,
    pub a: bool,
    pub d: bool,
}

impl KeyCommand {
    pub fn new(w: bool, s: bool, a: bool, d: bool) -> Self {
        Self {
            w,
            s,
            a,
            d,
        }
    }
}
