pub mod web_clipboard;

use bevy_app::{App, Plugin, PreStartup, PreUpdate};
use bevy_ecs::system::{ResMut, Resource};
use logging::info;

use crate::wasm::web_clipboard::WebClipboard;

#[derive(Default)]
pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClipboardManager>()
            .init_non_send_resource::<web_clipboard::SubscribedEvents>()
            .add_systems(PreStartup, web_clipboard::startup_setup_web_events)
            .add_systems(PreUpdate, handle_clipboard_events);
    }
}

#[derive(Resource)]
pub struct ClipboardManager {
    clipboard: WebClipboard,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self {
            clipboard: WebClipboard::default(),
        }
    }
}

impl ClipboardManager {
    /// Sets clipboard contents.
    pub fn set_contents(&mut self, contents: &str) {
        self.set_contents_impl(contents);
    }

    /// Sets the internal buffer of clipboard contents.
    /// This buffer is used to remember the contents of the last "Paste" event.
    pub fn set_contents_internal(&mut self, contents: &str) {
        self.clipboard.set_contents_internal(contents);
    }

    /// Gets clipboard contents. Returns [`None`] if clipboard provider is unavailable or returns an error.
    pub fn get_contents(&mut self) -> Option<String> {
        self.get_contents_impl()
    }

    /// Receives a clipboard event sent by the `copy`/`cut`/`paste` listeners.
    pub fn try_receive_clipboard_event(&self) -> Option<web_clipboard::WebClipboardEvent> {
        self.clipboard.try_receive_clipboard_event()
    }

    fn set_contents_impl(&mut self, contents: &str) {
        self.clipboard.set_contents(contents);
    }

    fn get_contents_impl(&mut self) -> Option<String> {
        self.clipboard.get_contents()
    }
}

fn handle_clipboard_events(mut clipboard_manager: ResMut<ClipboardManager>) {
    while let Some(event) = clipboard_manager.try_receive_clipboard_event() {
        match event {
            web_clipboard::WebClipboardEvent::Cut => {
                info!("received Cut");
            }
            web_clipboard::WebClipboardEvent::Copy => {
                info!("received Copy");
            }
            web_clipboard::WebClipboardEvent::Paste(contents) => {
                clipboard_manager.set_contents_internal(&contents);
                info!("received Paste: {:?}", contents);
            }
        }
    }
}
