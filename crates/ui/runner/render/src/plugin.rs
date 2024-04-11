use bevy_app::{App, Plugin};
use render_api::Draw;

use crate::draw_system;

// Plugin
pub struct UiRenderPlugin;

impl Plugin for UiRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Draw, draw_system::draw);
    }
}
