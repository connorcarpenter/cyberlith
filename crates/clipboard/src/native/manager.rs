use std::cell::{RefCell, RefMut};

use arboard::Clipboard;
use bevy_ecs::system::Resource;
use thread_local::ThreadLocal;

pub(crate) static mut THREAD_LOCAL_CLIPBOARD: Option<ThreadLocal<RefCell<Clipboard>>> = None;

#[derive(Resource)]
pub(crate) struct ClipboardManagerImpl {

}

impl Default for ClipboardManagerImpl {
    fn default() -> Self {

        unsafe {
            THREAD_LOCAL_CLIPBOARD = Some(ThreadLocal::new());
        }

        Self {

        }
    }
}

impl ClipboardManagerImpl {
    /// Sets clipboard contents.
    pub(crate) fn set_contents(&mut self, contents: &str) {
        let mut clipboard = Self::get_clipboard();
        if let Err(err) = clipboard.set_text(contents.to_owned()) {
            logging::error!("Failed to set clipboard contents: {:?}", err);
        }
    }

    pub(crate) fn get_clipboard() -> RefMut<'static, Clipboard> {
        unsafe {
            THREAD_LOCAL_CLIPBOARD.as_ref().unwrap().get_or(
                || Clipboard::new()
                    .map(RefCell::new)
                    .map_err(|err| {
                        logging::error!("Failed to initialize clipboard: {:?}", err);
                    })
                    .unwrap()
            ).borrow_mut()
        }
    }
}
