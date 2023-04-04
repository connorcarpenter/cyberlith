use bevy_app::{App, Plugin};
use bevy_ecs::schedule::IntoSystemSetConfig;

use render_api::RenderSet;

use crate::EguiDrawSet;

// Plugin
pub struct RenderEguiPlugin;

impl Plugin for RenderEguiPlugin {
    fn build(&self, app: &mut App) {
        app.configure_set(EguiDrawSet.after(RenderSet::Draw));
    }
}
