

use bevy_ecs::system::Resource;

use crate::ClipboardManagerImpl;

#[derive(Resource)]
pub struct ClipboardManager {
    pub(crate) inner: ClipboardManagerImpl,
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self {
            inner: ClipboardManagerImpl::default(),
        }
    }
}

impl ClipboardManager {
    /// Sets clipboard contents.
    pub fn set_contents(&mut self, contents: &str) {
        self.inner.set_contents(contents);
    }

    /// Gets clipboard contents. Returns [`None`] if clipboard provider is unavailable or returns an error.
    pub fn get_contents(&mut self) -> Option<String> {
        self.inner.get_contents()
    }
}
