use bevy_app::{App, Plugin};
use bevy_ecs::{
    component::Component,
    query::{With, Or},
    schedule::IntoSystemConfigs,
    system::{Commands, Local, Query, Res, ResMut, Resource},
};

use naia_bevy_client::{
    ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin, ReceiveEvents,
};

use math::{Quat, Vec3};
use render_api::{
    base::{Camera, Color, PbrMaterial, Texture2D, TriMesh, Viewport},
    shapes, AmbientLight, Assets, CameraComponent, ClearOperation, DirectionalLight, Handle,
    PointLight, RenderLayers, RenderObjectBundle, RenderTarget, Transform, Window,
};
use render_egui::{egui, EguiContext, EguiUserTextures, GUI, egui::{Modifiers, Ui, Widget}};

use editor_proto::protocol;

use crate::app::{network, ui};
use crate::app::ui::UiState;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add Naia Client Plugin
            // .add_plugin(NaiaClientPlugin::new(
            //     NaiaClientConfig::default(),
            //     protocol(),
            // ))
            // Startup Systems
            // .add_startup_system(network::init)
            .insert_resource(UiState::default())
            .add_startup_system(setup)
            .add_system(ui::main)
        // Receive Client Events
        // .add_systems(
        //     (
        //         network::connect_events,
        //         network::disconnect_events,
        //         network::reject_events,
        //         network::error_events,
        //     )
        //         .chain()
        //         .in_set(ReceiveEvents),
        // )
        // .add_system(step);
        ;
    }
}

fn setup(
    window: Res<Window>,
    mut commands: Commands,
) {
    // Ambient Light
    commands.insert_resource(AmbientLight::new(0.3, Color::WHITE));

    // Camera
    commands.spawn(CameraComponent::new(
        Camera::new_orthographic(
            window.viewport(),
            Vec3::new(50.0, 50.0, 50.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            50.0,
            0.1,
            1000.0,
        ),
        0,
        ClearOperation::default(),
        RenderTarget::Screen,
    ));
}