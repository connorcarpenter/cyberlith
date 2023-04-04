use bevy_app::{App, CoreSchedule, Plugin};
use bevy_ecs::schedule::{ExecutorKind, IntoSystemConfigs, Schedule};
use bevy_log::LogPlugin;

use naia_bevy_client::{
    ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin, ReceiveEvents,
};

use game_proto::protocol;
use render_api::{RenderApiPlugin, Window};

use crate::app::{
    plugin::GameClientPlugin,
    systems::{network, renderer},
};

pub fn build() -> App {
    let mut app = App::default();
    app
        // Bevy Plugins
        .add_plugin(LogPlugin::default())
        // Make all single-threaded
        .add_plugin(SingleThreadedPlugin)
        // Render API Plugin
        .add_plugin(RenderApiPlugin)
        // TODO: find out how to get window height & width
        .insert_resource(Window::new(1280, 720))
        // Add Renderer Plugin
        .add_plugin(renderer::RendererPlugin)
        // Add Naia Client Plugin
        .add_plugin(NaiaClientPlugin::new(
            NaiaClientConfig::default(),
            protocol(),
        ))
        // Add Game Client Plugin
        .add_plugin(GameClientPlugin)
        // Startup System
        .add_startup_system(network::init)
        // Receive Client Events
        .add_systems(
            (
                network::connect_events,
                network::disconnect_events,
                network::reject_events,
                network::error_events,
            )
                .chain()
                .in_set(ReceiveEvents),
        );
    app
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
