use bevy_app::{App, Startup, Update};
use bevy_ecs::change_detection::ResMut;

use game_engine::{
    render::{resources::WindowSettings, Draw},
    wait_for_finish, EnginePlugin,
    http::HttpClient,
    config::{GATEWAY_PORT, PUBLIC_IP_ADDR},
};

use crate::app::{
    resources::Global,
    systems::{
        draw, resize, scene, ui,
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
        .add_systems(Update, ui::ui_handle_events)
        // scene systems
        .add_systems(Startup, scene::scene_setup)
        .add_systems(Update, scene::scene_step)
        //.add_systems(Update, gamepad_testing::gamepad_testing_system)
        // viewport resize
        .add_systems(Update, resize::handle_viewport_resize)
        // draw
        .add_systems(Draw, draw::draw)

        // test gateway request
        .add_systems(Startup, test_request)
        .add_systems(Update, test_request_process)
    ;
    app.run();
}

#[allow(dead_code)]
pub async fn finished() {
    wait_for_finish().await;
}

use gateway_http_proto::{UserRegisterRequest, UserRegisterResponse};

// used as a system
pub fn test_request(
    mut http_client: ResMut<HttpClient>,
) {
    let request = UserRegisterRequest::new("charlie", "c@gmail.com", "12345");
    let key = http_client.send(PUBLIC_IP_ADDR, GATEWAY_PORT, request);
}

fn test_request_process(
    mut http_client: ResMut<HttpClient>,
) {
    // todo
}