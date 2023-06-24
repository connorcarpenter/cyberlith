use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::{IntoSystemConfig, IntoSystemSetConfig};

use render_api::RenderSet;

use crate::{EguiContext, EguiSet, EguiUserTextures, systems};

// Plugin
pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut App) {
        app
            // EGUI Specific
            .insert_resource(EguiContext::default())
            .insert_resource(EguiUserTextures::default())
            // System Sets
            .configure_set(
                EguiSet::PreUpdate
                    .after(CoreSet::First)
                    .before(CoreSet::FirstFlush))
            .configure_set(
                EguiSet::PostUpdate
                    .after(CoreSet::PostUpdate)
                    .before(CoreSet::PostUpdateFlush))
            .configure_set(
                EguiSet::Sync
                    .after(RenderSet::Sync)
                    .before(RenderSet::SyncFlush),
            )
            .configure_set(
                EguiSet::Draw
                    .after(RenderSet::Draw))
            // Systems
            .add_startup_system(systems::startup)
            .add_system(systems::pre_update.in_base_set(EguiSet::PreUpdate))
            .add_system(systems::post_update.in_base_set(EguiSet::PostUpdate))
            .add_system(systems::sync.in_base_set(EguiSet::Sync))
            .add_system(systems::draw.in_base_set(EguiSet::Draw));
    }
}
