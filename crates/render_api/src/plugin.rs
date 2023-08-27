use bevy_app::{App, Last, MainScheduleOrder, Plugin};
use bevy_ecs::schedule::{ExecutorKind, Schedule};

use crate::{
    assets::Assets,
    base::{CpuMaterial, CpuMesh, CpuTexture2D},
    base_set::{RenderDraw, RenderSync},
    Window,
};

pub struct RenderApiPlugin;

impl Plugin for RenderApiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .insert_resource(Assets::<CpuMesh>::default())
            .insert_resource(Assets::<CpuMaterial>::default())
            .insert_resource(Assets::<CpuTexture2D>::default())
            // TODO: figure out how to set the correct window here ...
            .insert_resource(Window::default());

        // Schedules
        app.init_schedule(RenderSync);
        app.init_schedule(RenderDraw);

        let make_single_threaded_fn = |schedule: &mut Schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        };
        app.edit_schedule(RenderSync, make_single_threaded_fn.clone());
        app.edit_schedule(RenderDraw, make_single_threaded_fn.clone());

        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(Last, RenderSync);
        order.insert_after(RenderSync, RenderDraw);
    }
}
