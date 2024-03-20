use bevy_ecs::system::Commands;

use game_engine::{
    render::{
        base::Color,
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation,
            OrthographicProjection, Projection,
            RenderTarget,
        },
    },
};

use crate::app::global::Global;

pub fn setup_scene(
    mut commands: Commands
) {
    // ambient light
    commands
        .spawn(AmbientLight::new(1.0, Color::WHITE));

    // camera
    let camera_id = commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: None,
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Screen,
                ..Default::default()
            },
            projection: Projection::Orthographic(OrthographicProjection {
                near: 0.0,
                far: 2000.0,
            }),
            ..Default::default()
        })
        .id();

    commands.insert_resource(Global::new(camera_id));
}
