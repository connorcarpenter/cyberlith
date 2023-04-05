use math::Vec2;

/// An event that is sent whenever the user's cursor enters a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorEntered;

/// An event that is sent whenever the user's cursor leaves a window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CursorLeft;

/// An event reporting that the mouse cursor has moved inside a window.
#[derive(Debug, Clone, PartialEq)]
pub struct CursorMoved {
    /// The cursor position in logical pixels.
    pub position: Vec2,
}

/// An event that is sent whenever a window receives a character from the OS or underlying system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReceivedCharacter {
    /// Received character.
    pub char: char,
}

/// An event that indicates a window has received or lost focus.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowFocused {
    /// Whether it was focused (true) or lost focused (false).
    pub focused: bool,
}

/// An event that is sent whenever a new window is created.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowCreated;

/// An event that indicates the window should redraw, even if its control flow is set to `Wait` and
/// there have been no window events.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RequestRedraw;