use input::{IncomingEvent, Modifiers, MouseButton};

pub(crate) struct RenderLoopContext {
    pub(crate) last_time: instant::Instant,
    pub(crate) accumulated_time: f64,
    pub(crate) events: Vec<IncomingEvent>,
    pub(crate) cursor_pos: Option<(f64, f64)>,
    pub(crate) finger_id: Option<u64>,
    pub(crate) secondary_cursor_pos: Option<(f64, f64)>,
    pub(crate) secondary_finger_id: Option<u64>,
    pub(crate) modifiers: Modifiers,
    pub(crate) first_frame: bool,
    pub(crate) mouse_pressed: Option<MouseButton>,
}

impl RenderLoopContext {
    pub fn new() -> Self {
        Self {
            last_time: instant::Instant::now(),
            accumulated_time: 0.0,
            events: Vec::new(),
            cursor_pos: None,
            finger_id: None,
            secondary_cursor_pos: None,
            secondary_finger_id: None,
            modifiers: Modifiers::default(),
            first_frame: true,
            mouse_pressed: None,
        }
    }
}
