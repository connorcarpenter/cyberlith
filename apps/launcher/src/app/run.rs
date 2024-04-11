use bevy_app::{App, Startup, Update};

use game_engine::{
    render::{resources::WindowSettings, Draw},
    wait_for_finish, EnginePlugin,
};

use crate::app::{
    resources::Global,
    systems::{draw, resize, scene, ui, backend},
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
        .add_event::<LoginButtonClickedEvent>()
        .add_event::<RegisterButtonClickedEvent>()
        .add_event::<SubmitButtonClickedEvent>()
        // ui systems
        .add_systems(Startup, ui::ui_setup)
        .add_systems(Update, ui::ui_handle_events)
        .add_systems(Update, backend::backend_process_responses)
        // scene systems
        .add_systems(Startup, scene::scene_setup)
        .add_systems(Update, scene::scene_step)
        //.add_systems(Update, gamepad_testing::gamepad_testing_system)
        // viewport resize
        .add_systems(Update, resize::handle_viewport_resize)
        // draw
        .add_systems(Draw, draw::draw);
    app.run();
}

#[allow(dead_code)]
pub async fn finished() {
    wait_for_finish().await;
}

use crate::app::resources::{
    LoginButtonClickedEvent, RegisterButtonClickedEvent, SubmitButtonClickedEvent,
};