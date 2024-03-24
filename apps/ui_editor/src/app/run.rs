use bevy_app::{App, Startup, Update};

use game_engine::{
    render::{resources::WindowSettings, Draw},
    EnginePlugin,
};

use super::{draw, resize, scene, ui};
use crate::app::ui::SubmitButtonEvent;

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
        .add_event::<SubmitButtonEvent>()
        // ui systems
        .add_systems(Startup, ui::ui_setup)
        .add_systems(Update, ui::ui_update)
        .add_systems(Update, ui::ui_handle_events)
        // scene systems
        .add_systems(Startup, scene::scene_setup)
        // viewport resize
        .add_systems(Update, resize::handle_viewport_resize)
        // draw
        .add_systems(Draw, draw::draw);
    app.run();
}
