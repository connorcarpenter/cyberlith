use bevy_app::{App, Plugin};

use crate::{AppState, ViewportResizeEvent};

pub struct CommonPlugin;

impl Plugin for CommonPlugin {
    fn build(&self, app: &mut App) {
        app
            // states
            .insert_state(AppState::Loading)

            // resize window event
            .add_event::<ViewportResizeEvent>();
    }
}