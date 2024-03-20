use bevy_app::{App, Startup, Update};

use game_engine::{
    render::{resources::WindowSettings, Draw},
    EnginePlugin,
};

use crate::app::ui::{ContinueButtonEvent, StartButtonEvent};
use super::{scene, ui, draw, resize};

pub fn run() {
    let mut app = App::default();

    app.add_plugins(EnginePlugin)
        // Add Window Settings Plugin
        .insert_resource(WindowSettings {
            title: "UI Editor".to_string(),
            min_size: (320, 180),
            max_size: None,
            ..Default::default()
        })
        // events
        .add_event::<StartButtonEvent>()
        .add_event::<ContinueButtonEvent>()
        // systems
        .add_systems(Startup, scene::setup_scene)
        .add_systems(Startup, ui::setup_ui)
        .add_systems(Update, ui::handle_events)
        .add_systems(Draw, draw::scene_draw)
        .add_systems(Update, resize::handle_viewport_resize);
    app.run();
}
