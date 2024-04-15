
use bevy_ecs::system::{Resource};

use crate::wasm::web_clipboard::{WebClipboard, WebClipboardEvent};

#[derive(Resource)]
pub(crate) struct ClipboardManagerImpl {
    pub(crate) clipboard: WebClipboard,
}

impl Default for ClipboardManagerImpl {
    fn default() -> Self {
        Self {
            clipboard: WebClipboard::default(),
        }
    }
}

impl ClipboardManagerImpl {
    /// Sets clipboard contents.
    pub(crate) fn set_contents(&mut self, contents: &str) {
        self.set_contents_impl(contents);
    }

    /// Gets clipboard contents. Returns [`None`] if clipboard provider is unavailable or returns an error.
    pub(crate) fn get_contents(&mut self) -> Option<String> {
        self.get_contents_impl()
    }

    /// Sets the internal buffer of clipboard contents.
    /// This buffer is used to remember the contents of the last "Paste" event.
    pub(crate) fn set_contents_internal(&mut self, contents: &str) {
        self.clipboard.set_contents_internal(contents);
    }

    /// Receives a clipboard event sent by the `copy`/`cut`/`paste` listeners.
    pub fn try_receive_clipboard_event(&self) -> Option<WebClipboardEvent> {
        self.clipboard.try_receive_clipboard_event()
    }

    fn set_contents_impl(&mut self, contents: &str) {
        self.clipboard.set_contents(contents);
    }

    fn get_contents_impl(&mut self) -> Option<String> {
        self.clipboard.get_contents()
    }
}
