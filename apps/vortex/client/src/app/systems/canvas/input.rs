use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{Query, ResMut},
};

use input::Input;
use render_api::components::{Camera, Projection, Transform, Visibility};

use crate::app::{components::Vertex2d, resources::canvas_manager::CanvasManager};

pub fn input(
    mut canvas_manager: ResMut<CanvasManager>,
    mut input: ResMut<Input>,
    mut transform_q: Query<&mut Transform>,
    mut camera_q: Query<(&mut Camera, &mut Projection)>,
    mut visibility_q: Query<&mut Visibility>,
    vertex_2d_q: Query<Entity, With<Vertex2d>>,
) {
    canvas_manager.update_input(
        &mut input,
        &mut transform_q,
        &mut camera_q,
        &mut visibility_q,
        &vertex_2d_q,
    );
}
