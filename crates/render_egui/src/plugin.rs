use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::{IntoSystemConfig, IntoSystemSetConfig};

use render_api::{RenderApiPlugin, RenderSet};
use render_glow::RenderGlowPlugin;

use crate::{systems, EguiContext, EguiSet, EguiUserTextures, GUI};

// Plugin
pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Renderer Specific
            .add_plugin(RenderApiPlugin)
            .add_plugin(RenderGlowPlugin)
            // EGUI Specific
            .insert_resource(EguiContext::default())
            // System Sets
            .configure_set(EguiSet::PreUpdate.before(CoreSet::Update))
            .configure_set(EguiSet::PostUpdate.after(CoreSet::Update))
            .configure_set(EguiSet::Draw.after(EguiSet::PostUpdate))
            // Systems
            .add_startup_system(systems::startup)
            .add_system(systems::pre_update.in_base_set(EguiSet::PreUpdate))
            .add_system(systems::post_update.in_base_set(EguiSet::PostUpdate))
            .add_system(systems::draw.in_base_set(EguiSet::Draw));
    }
}
