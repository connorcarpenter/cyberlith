
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use logging::{warn, info};

use crate::wasm::string_from_js_value;

pub(crate) struct ClipboardManagerImpl;

impl ClipboardManagerImpl {
    pub(crate) fn init() {
        // Nothing to do here
    }
}

impl ClipboardManagerImpl {
    /// Sets clipboard contents.
    pub(crate) fn set_contents(contents: &str) {
        clipboard_set(contents.to_owned());
    }
}

/// Sets contents of the clipboard via the Web API.
fn clipboard_set(contents: String) {
    spawn_local(async move {
        let Some(window) = window() else {
            warn!("Failed to access the window object");
            return;
        };

        let nav = window.navigator();
        let Some(clipboard) = nav.clipboard() else {
            warn!("Failed to access clipboard");
            return;
        };

        info!("writing to clipboard: {}", contents);
        let promise = clipboard.write_text(&contents);
        if let Err(err) = wasm_bindgen_futures::JsFuture::from(promise).await {
            warn!(
                "Failed to write to clipboard: {}",
                string_from_js_value(&err)
            );
        }
    });
}
