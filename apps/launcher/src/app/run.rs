use bevy_app::{App, Startup, Update};

use game_engine::{
    render::{resources::WindowSettings, Draw},
    wait_for_finish, EnginePlugin,
};

use crate::app::{
    resources::Global,
    systems::{
        draw, gamepad, resize, scene, ui,
        ui::{ContinueButtonEvent, StartButtonEvent},
    },
};

pub fn run() {
    let mut app = App::default();

    app.add_plugins(EnginePlugin)
        // Add Window Settings Plugin
        .insert_resource(WindowSettings {
            title: "Cyberlith Launcher".to_string(),
            min_size: (320, 180),
            max_size: None,
            ..Default::default()
        })
        // global resource
        .init_resource::<Global>()
        // events
        .add_event::<StartButtonEvent>()
        .add_event::<ContinueButtonEvent>()
        // ui systems
        .add_systems(Startup, ui::ui_setup)
        .add_systems(Update, ui::handle_events)
        // scene systems
        .add_systems(Startup, scene::scene_setup)
        .add_systems(Update, scene::scene_step)
        .add_systems(Update, gamepad::gamepad_system)
        // viewport resize
        .add_systems(Update, resize::handle_viewport_resize)
        .add_systems(Draw, draw::draw);
    app.run();
}

#[allow(dead_code)]
pub async fn finished() {
    wait_for_finish().await;
}
