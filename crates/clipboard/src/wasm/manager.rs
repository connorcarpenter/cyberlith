
use bevy_ecs::system::{Resource};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use logging::{warn, info};

#[derive(Resource)]
pub(crate) struct ClipboardManagerImpl {

}

impl Default for ClipboardManagerImpl {
    fn default() -> Self {
        Self {

        }
    }
}

impl ClipboardManagerImpl {
    /// Sets clipboard contents.
    pub(crate) fn set_contents(&mut self, contents: &str) {
        clipboard_set(contents.to_owned());
    }

    /// Gets clipboard contents. Returns [`None`] if clipboard provider is unavailable or returns an error.
    pub(crate) fn get_contents(&mut self) -> Option<String> {
        clipboard_get()
    }
}

/// Sets contents of the clipboard via the Web API.
fn clipboard_get() -> Option<String> {

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

        info!("reading from clipboard");
        let promise = clipboard.read_text();
        match wasm_bindgen_futures::JsFuture::from(promise).await {
            Ok(value) => {
                match value.as_string() {
                    Some(contents) => {
                        info!("read from clipboard: {}", contents);
                        //sender.send(Some(contents)).unwrap();
                        return;
                    }
                    None => {
                        warn!("Failed to read from clipboard: empty value");
                        return;
                    }
                }
            },
            Err(err) => {
                warn!(
                    "Failed to read from clipboard: {}",
                    string_from_js_value(&err)
                );
                return;
            }
        }

        //sender.send(None).unwrap();
    });

    None
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

fn string_from_js_value(value: &JsValue) -> String {
    value.as_string().unwrap_or_else(|| format!("{value:#?}"))
}
