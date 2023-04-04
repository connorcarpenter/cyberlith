use bevy_app::{App, CoreSchedule, Plugin};
use bevy_ecs::schedule::{ExecutorKind, IntoSystemConfig, Schedule};

use render_api::RenderSet;

use crate::{draw::draw, runner::three_d_runner, sync::SyncPlugin};

cfg_if! {
    if #[cfg(feature = "editor")] {
        use crate::egui::EguiPlugin;
    }
}

pub struct RenderGlowPlugin;

impl Plugin for RenderGlowPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugin(SyncPlugin)
            .add_plugin(SingleThreadedPlugin)
            // Runner for Three-D integration
            .set_runner(three_d_runner)
            // Systems
            .add_system(draw.in_base_set(RenderSet::Draw));
        maybe_editor_plugin(app);
    }
}

fn maybe_editor_plugin(app: &mut App) {
    cfg_if! {
        if #[cfg(feature = "editor")] {
            app.add_plugin(EguiPlugin);
        }
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
