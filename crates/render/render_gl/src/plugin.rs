use std::sync::{Arc, RwLock};

use bevy_app::{
    App, First, Last, Main, MainScheduleOrder, Plugin, PostUpdate, PreStartup, PreUpdate,
    RunFixedMainLoop, StateTransition, Update,
};
use bevy_ecs::schedule::{ExecutorKind, Schedule};

use render_api::{Render, Window};

use crate::{
    base_set::GlInput,
    input,
    render::render,
    runner::{runner_func, StopSignal, STOP_SIGNAL},
    sync::SyncPlugin,
    window,
};

pub struct RenderGlPlugin;

impl Plugin for RenderGlPlugin {
    fn build(&self, app: &mut App) {
        unsafe {
            STOP_SIGNAL = Some(Arc::new(RwLock::new(StopSignal { stopped: false })));
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
            .add_systems(GlInput, input::run)
            .add_systems(Render, render);

        let mut order = app.world.resource_mut::<MainScheduleOrder>();
        order.insert_after(PreUpdate, GlInput);

        let make_single_threaded_fn = |schedule: &mut Schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        };
        app.edit_schedule(GlInput, make_single_threaded_fn.clone());
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
