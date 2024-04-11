use bevy_ecs::system::Commands;

use game_engine::render::components::{CameraBundle, Viewport};

use crate::app::global::Global;

pub fn startup(mut commands: Commands) {
    let camera_entity = commands
        .spawn(CameraBundle::default_3d_perspective(
            &Viewport::new_at_origin(0, 0),
        ))
        .id();

    let mut global = Global::new(camera_entity);

    commands.insert_resource(global);
}
