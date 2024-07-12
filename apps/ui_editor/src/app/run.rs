use bevy_app::{App, Startup, Update};

use game_engine::{
    render::{resources::WindowSettings, Draw},
    EnginePlugin, kernel::executor,
};

use super::{draw, resize, ui};
use crate::app::{scroll::scroll_events, ui::SubmitButtonEvent};

pub fn run() {
    logging::initialize();
    executor::setup(1, 1);

    let mut app = App::default();

    app.add_plugins(EnginePlugin::new(None))
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
        .add_systems(Startup, ui::setup)
        .add_systems(Update, ui::handle_events)
        .add_systems(Update, scroll_events)
        // viewport resize
        .add_systems(Update, resize::handle_viewport_resize)
        // draw
        .add_systems(Draw, draw::draw);
    app.run();
}
