use bevy_app::{App, Last, MainScheduleOrder, Plugin};
use bevy_ecs::schedule::{ExecutorKind, Schedule};

use storage::Storage;

use crate::{
    base::{CpuMaterial, CpuMesh, CpuSkin, CpuTexture2D},
    base_set::{Draw, RenderSync},
    resources::{RenderFrame, Time},
    Render,
};

pub struct RenderApiPlugin;

impl Plugin for RenderApiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<Storage<CpuMesh>>()
            .init_resource::<Storage<CpuMaterial>>()
            .init_resource::<Storage<CpuTexture2D>>()
            .init_resource::<Storage<CpuSkin>>()
            .init_resource::<RenderFrame>()
            .init_resource::<Time>();

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

        let mut order = app.world_mut().resource_mut::<MainScheduleOrder>();
        order.insert_after(Last, Draw);

        order.insert_after(Draw, RenderSync);
        order.insert_after(RenderSync, Render);
    }
}
