
use bevy_app::{App, Plugin};

use crate::{ClipboardManager, ClipboardPluginImpl};

#[derive(Default)]
pub struct ClipboardPlugin;

impl Plugin for ClipboardPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ClipboardManager>()
            .add_plugins(ClipboardPluginImpl);
    }
}