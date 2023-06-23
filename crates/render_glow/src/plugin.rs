use bevy_app::{App, CoreSchedule, Plugin};
use bevy_ecs::schedule::{ExecutorKind, IntoSystemConfig, Schedule};

use render_api::RenderSet;

use crate::{draw::draw, runner::runner_func, sync::SyncPlugin};

pub struct RenderGlowPlugin;

impl Plugin for RenderGlowPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugin(SyncPlugin)
            .add_plugin(SingleThreadedPlugin)
            // Runner
            .set_runner(runner_func)
            // Systems
            .add_system(draw.in_base_set(RenderSet::Draw));
    }
}

struct SingleThreadedPlugin;

impl Plugin for SingleThreadedPlugin {
    fn build(&self, app: &mut App) {
        let make_single_threaded_fn = |schedule: &mut Schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        };
        app.edit_schedule(CoreSchedule::Outer, make_single_threaded_fn.clone());
        app.edit_schedule(CoreSchedule::Startup, make_single_threaded_fn.clone());
        app.edit_schedule(CoreSchedule::Main, make_single_threaded_fn.clone());
        app.edit_schedule(CoreSchedule::FixedUpdate, make_single_threaded_fn);
    }
}
