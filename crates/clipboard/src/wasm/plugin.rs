
use bevy_app::{App, Plugin, PreStartup, PreUpdate};
use bevy_ecs::system::{ResMut};
use logging::info;

use crate::{wasm::web_clipboard::{startup_setup_web_events, SubscribedEvents, WebClipboardEvent}, ClipboardManager};

#[derive(Default)]
pub(crate) struct ClipboardPluginImpl;

impl Plugin for ClipboardPluginImpl {
    fn build(&self, app: &mut App) {
        app
            .init_non_send_resource::<SubscribedEvents>()
            .add_systems(PreStartup, startup_setup_web_events)
            .add_systems(PreUpdate, handle_clipboard_events);
    }
}

fn handle_clipboard_events(mut clipboard_manager: ResMut<ClipboardManager>) {
    while let Some(event) = clipboard_manager.inner.try_receive_clipboard_event() {
        match event {
            WebClipboardEvent::Cut => {
                info!("received Cut");
            }
            WebClipboardEvent::Copy => {
                info!("received Copy");
            }
            WebClipboardEvent::Paste(contents) => {
                clipboard_manager.inner.set_contents_internal(&contents);
                info!("received Paste: {:?}", contents);
            }
        }
    }
}
