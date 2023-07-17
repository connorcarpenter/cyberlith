use bevy_ecs::{entity::Entity, query::{With, Without}, system::{Query, ResMut}};

use input::Input;
use render_api::components::{Camera, Projection, Transform, Visibility};

use crate::app::{components::{HoverCircle, Vertex2d}, resources::canvas_manager::CanvasManager};

pub fn input(
    mut canvas_manager: ResMut<CanvasManager>,
    mut input: ResMut<Input>,
    mut camera_query: Query<(&mut Camera, &mut Transform, &mut Projection), (Without<HoverCircle>, Without<Vertex2d>)>,
    mut hover_query: Query<(&mut Transform, &mut Visibility), (With<HoverCircle>, Without<Vertex2d>)>,
    vertex_2d_query: Query<(Entity, &Transform), With<Vertex2d>>,
) {
    canvas_manager.update_input(&mut input, &mut camera_query, &mut hover_query, &vertex_2d_query);
}