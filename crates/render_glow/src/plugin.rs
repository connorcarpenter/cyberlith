use bevy_app::{App, First, Last, Main, MainScheduleOrder, Plugin, PostUpdate, PreUpdate, RunFixedUpdateLoop, StateTransition, Update};
use bevy_ecs::schedule::{ExecutorKind, Schedule};

use render_api::RenderDraw;

use crate::{base_set::GlowInput, draw::draw, input, runner::runner_func, sync::SyncPlugin};

pub struct RenderGlowPlugin;

impl Plugin for RenderGlowPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugins(SyncPlugin)
            .add_plugins(SingleThreadedPlugin)
            // Runner
            .set_runner(runner_func)
            // Systems
            .add_systems(GlowInput, input::run)
            .add_systems(RenderDraw, draw);

        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(PreUpdate, GlowInput);

        let make_single_threaded_fn = |schedule: &mut Schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        };
        app.edit_schedule(GlowInput, make_single_threaded_fn.clone());
    }
}

struct SingleThreadedPlugin;

impl Plugin for SingleThreadedPlugin {
    fn build(&self, app: &mut App) {
        let make_single_threaded_fn = |schedule: &mut Schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        };
        app.edit_schedule(Main, make_single_threaded_fn.clone());
        app.edit_schedule(First, make_single_threaded_fn.clone());
        app.edit_schedule(PreUpdate, make_single_threaded_fn.clone());
        app.edit_schedule(StateTransition, make_single_threaded_fn.clone());
        app.edit_schedule(RunFixedUpdateLoop, make_single_threaded_fn.clone());
        app.edit_schedule(Update, make_single_threaded_fn.clone());
        app.edit_schedule(PostUpdate, make_single_threaded_fn.clone());
        app.edit_schedule(Last, make_single_threaded_fn);
    }
}
