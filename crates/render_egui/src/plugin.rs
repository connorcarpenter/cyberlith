use bevy_app::{App, CoreSet, Plugin};
use bevy_ecs::schedule::{IntoSystemConfig, IntoSystemSetConfig};

use render_api::{RenderApiPlugin, RenderSet};
use render_glow::RenderGlowPlugin;

use crate::resources::{EguiInput, EguiOutput, EguiRenderOutput, WindowSize};
use crate::{
    draw,
    resources::{
        EguiContext, EguiManagedTextures, EguiMousePosition, EguiSettings, EguiUserTextures,
    },
    systems, EguiSet,
};

// Plugin
pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenderApiPlugin)
            .add_plugin(RenderGlowPlugin)
            // Global
            .init_resource::<EguiSettings>()
            .init_resource::<EguiManagedTextures>()
            .init_resource::<EguiUserTextures>()
            // Window
            .init_resource::<EguiContext>()
            .init_resource::<EguiMousePosition>()
            .init_resource::<EguiRenderOutput>()
            .init_resource::<EguiInput>()
            .init_resource::<EguiOutput>()
            .init_resource::<WindowSize>()
            // Systems
            .add_system(
                systems::update_window_context
                    .in_set(EguiSet::InitContexts)
                    .in_base_set(CoreSet::PreUpdate),
            )
            .add_system(
                systems::process_input
                    .in_set(EguiSet::ProcessInput)
                    .after(EguiSet::InitContexts)
                    .in_base_set(CoreSet::PreUpdate),
            );
    }
}
