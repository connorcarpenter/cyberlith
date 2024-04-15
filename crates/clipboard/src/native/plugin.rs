
use bevy_app::{App, Plugin};

#[derive(Default)]
pub(crate) struct ClipboardPluginImpl;

impl Plugin for ClipboardPluginImpl {
    fn build(&self, _app: &mut App) {
        // nothing extra on native
    }
}