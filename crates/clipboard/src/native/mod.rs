use std::cell::{RefCell, RefMut};

use arboard::Clipboard;

use bevy_app::{App, Plugin};
use bevy_ecs::system::Resource;

#[derive(Default)]
pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ClipboardManager>();
    }
}

#[derive(Resource)]
pub struct ClipboardManager {
    clipboard: thread_local::ThreadLocal<Option<RefCell<Clipboard>>>,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self {
            clipboard: thread_local::ThreadLocal::default(),
        }
    }
}

impl ClipboardManager {
    /// Sets clipboard contents.
    pub fn set_contents(&mut self, contents: &str) {
        self.set_contents_impl(contents);
    }

    /// Gets clipboard contents. Returns [`None`] if clipboard provider is unavailable or returns an error.
    pub fn get_contents(&mut self) -> Option<String> {
        self.get_contents_impl()
    }

    fn set_contents_impl(&mut self, contents: &str) {
        if let Some(mut clipboard) = self.get() {
            if let Err(err) = clipboard.set_text(contents.to_owned()) {
                log::error!("Failed to set clipboard contents: {:?}", err);
            }
        }
    }

    fn get_contents_impl(&mut self) -> Option<String> {
        if let Some(mut clipboard) = self.get() {
            match clipboard.get_text() {
                Ok(contents) => return Some(contents),
                Err(err) => log::error!("Failed to get clipboard contents: {:?}", err),
            }
        };
        None
    }

    fn get(&self) -> Option<RefMut<Clipboard>> {
        self.clipboard
            .get_or(|| {
                Clipboard::new()
                    .map(RefCell::new)
                    .map_err(|err| {
                        log::error!("Failed to initialize clipboard: {:?}", err);
                    })
                    .ok()
            })
            .as_ref()
            .map(|cell| cell.borrow_mut())
    }
}
