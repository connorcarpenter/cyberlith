use std::cell::{RefCell, RefMut};

use arboard::Clipboard;

use bevy_ecs::system::Resource;

#[derive(Resource)]
pub(crate) struct ClipboardManagerImpl {
    clipboard: thread_local::ThreadLocal<Option<RefCell<Clipboard>>>,
}

impl Default for ClipboardManagerImpl {
    fn default() -> Self {
        Self {
            clipboard: thread_local::ThreadLocal::default(),
        }
    }
}

impl ClipboardManagerImpl {
    /// Sets clipboard contents.
    pub(crate) fn set_contents(&mut self, contents: &str) {
        if let Some(mut clipboard) = self.get() {
            if let Err(err) = clipboard.set_text(contents.to_owned()) {
                logging::error!("Failed to set clipboard contents: {:?}", err);
            }
        }
    }

    /// Gets clipboard contents. Returns [`None`] if clipboard provider is unavailable or returns an error.
    pub(crate) fn get_contents(&mut self) -> Option<String> {
        if let Some(mut clipboard) = self.get() {
            match clipboard.get_text() {
                Ok(contents) => return Some(contents),
                Err(err) => logging::error!("Failed to get clipboard contents: {:?}", err),
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
                        logging::error!("Failed to initialize clipboard: {:?}", err);
                    })
                    .ok()
            })
            .as_ref()
            .map(|cell| cell.borrow_mut())
    }
}
