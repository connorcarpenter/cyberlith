use bevy_app::{App, First, MainScheduleOrder, Plugin, PostUpdate, Startup};
use bevy_ecs::schedule::{IntoSystemSetConfig};

use render_api::{RenderDraw, RenderSync};

use crate::{systems, EguiContext, EguiPreUpdate, EguiPostUpdate, EguiDraw, EguiSync, EguiUserTextures};

// Plugin
pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut App) {
        app
            // EGUI Specific
            .insert_resource(EguiContext::default())
            .insert_resource(EguiUserTextures::default())
            // Systems
            .add_systems(Startup, systems::startup)
            .add_systems(EguiPreUpdate, systems::pre_update)
            .add_systems(EguiPostUpdate, systems::post_update)
            .add_systems(EguiSync, systems::sync)
            .add_systems(EguiDraw, systems::draw);

        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(First, EguiPreUpdate);
        order.insert_after(PostUpdate, EguiPostUpdate);
        order.insert_after(RenderSync, EguiSync);
        order.insert_after(RenderDraw, EguiDraw);
    }
}
