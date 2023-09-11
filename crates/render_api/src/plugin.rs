use bevy_app::{App, Last, MainScheduleOrder, Plugin};
use bevy_ecs::schedule::{ExecutorKind, Schedule};

use crate::{
    components::{AmbientLight, DirectionalLight},
    assets::Assets,
    base::{CpuMaterial, CpuMesh, CpuTexture2D},
    base_set::{Draw, RenderSync},
    resources::RenderFrame,
    Render,
};

pub struct RenderApiPlugin;

impl Plugin for RenderApiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<Assets<CpuMesh>>()
            .init_resource::<Assets<CpuMaterial>>()
            .init_resource::<Assets<CpuTexture2D>>()
            .init_resource::<Assets<AmbientLight>>()
            .init_resource::<Assets<DirectionalLight>>()
            .init_resource::<RenderFrame>();

        // Schedules
        app.init_schedule(RenderSync);
        app.init_schedule(Draw);
        app.init_schedule(Render);

        let make_single_threaded_fn = |schedule: &mut Schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        };
        app.edit_schedule(RenderSync, make_single_threaded_fn.clone());
        app.edit_schedule(Draw, make_single_threaded_fn.clone());
        app.edit_schedule(Render, make_single_threaded_fn.clone());

        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(Last, RenderSync);
        order.insert_after(RenderSync, Draw);
        order.insert_after(Draw, Render);
    }
}
