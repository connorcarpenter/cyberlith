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
            .insert_resource(EguiUserTextures::default())
            // System Sets
            .configure_set(EguiSet::PreUpdate.before(CoreSet::First))
            .configure_set(EguiSet::PostUpdate.after(CoreSet::LastFlush))
            .configure_set(EguiSet::Sync.after(RenderSet::Sync).before(RenderSet::SyncFlush))
            .configure_set(EguiSet::Draw.after(RenderSet::Draw))
            // Systems
            .add_startup_system(systems::startup)
            .add_system(systems::pre_update.in_base_set(EguiSet::PreUpdate))
            .add_system(systems::post_update.in_base_set(EguiSet::PostUpdate))
            .add_system(systems::sync.in_base_set(EguiSet::Sync))
            .add_system(systems::draw.in_base_set(EguiSet::Draw));
    }
}
