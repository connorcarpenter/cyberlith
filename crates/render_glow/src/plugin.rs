use bevy_app::{App, CoreSchedule, CoreSet, Plugin};
use bevy_ecs::schedule::{ExecutorKind, IntoSystemConfig, IntoSystemSetConfig, Schedule};

use render_api::RenderSet;

use crate::{draw::draw, input, runner::runner_func, sync::SyncPlugin};
use crate::base_set::GlowSet;

pub struct RenderGlowPlugin;

impl Plugin for RenderGlowPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugin(SyncPlugin)
            .add_plugin(SingleThreadedPlugin)
            // Runner
            .set_runner(runner_func)
            // Sets
            // System Sets
            .configure_set(
                GlowSet::Input
                    .after(CoreSet::PreUpdate)
                    .before(CoreSet::PreUpdateFlush),
            )
            // Systems
            .add_system(input::run.in_base_set(GlowSet::Input))
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
