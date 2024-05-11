use bevy_app::{
    App, First, Last, Main, Plugin, PostUpdate, PreStartup, PreUpdate, RunFixedMainLoop,
    StateTransition, Update,
};
use bevy_ecs::schedule::{ExecutorKind, Schedule};

use clipboard::ClipboardPlugin;
use kernel::KernelPlugin;
use render_api::{Render, Window};

use crate::{input, exit_system, render::render, runner::runner_func, sync::SyncPlugin, window};

pub struct RenderGlPlugin;

impl Plugin for RenderGlPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<ClipboardPlugin>() {
            app.add_plugins(ClipboardPlugin);
        }
        if !app.is_plugin_added::<KernelPlugin>() {
            panic!("KernelPlugin required for RenderGlPlugin.");
        }

        app
            // Plugins
            .add_plugins(SyncPlugin)
            .add_plugins(SingleThreadedPlugin)
            // Runner
            .set_runner(runner_func)
            // Resources
            .insert_resource(Window::default())
            // Systems
            .add_systems(PreStartup, window::sync)
            .add_systems(First, window::sync)
            .add_systems(PreUpdate, input::run)
            .add_systems(PostUpdate, input::update_cursor)
            .add_systems(PostUpdate, exit_system::app_exit)
            .add_systems(Render, render);
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
        app.edit_schedule(RunFixedMainLoop, make_single_threaded_fn.clone());
        app.edit_schedule(Update, make_single_threaded_fn.clone());
        app.edit_schedule(PostUpdate, make_single_threaded_fn.clone());
        app.edit_schedule(Last, make_single_threaded_fn);
    }
}
