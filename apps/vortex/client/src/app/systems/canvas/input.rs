use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{Commands, Query, ResMut},
};
use naia_bevy_client::Client;

use input::Input;
use render_api::{Assets, base::{CpuMaterial, CpuMesh}, components::{Camera, Projection, Transform, Visibility}};
use vortex_proto::components::Vertex3d;

use crate::app::{
    components::{Edge2d, Vertex2d},
    resources::canvas_manager::CanvasManager,
};

pub fn input(
    mut commands: Commands,
    mut client: Client,
    mut canvas_manager: ResMut<CanvasManager>,
    mut input: ResMut<Input>,
    mut transform_q: Query<&mut Transform>,
    mut camera_q: Query<(&mut Camera, &mut Projection)>,
    mut visibility_q: Query<&mut Visibility>,
    mut vertex_3d_q: Query<&mut Vertex3d>,
    vertex_2d_q: Query<Entity, With<Vertex2d>>,
    edge_2d_q: Query<Entity, With<Edge2d>>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
) {
    canvas_manager.update_input(
        &mut commands,
        &mut client,
        &mut input,
        &mut transform_q,
        &mut camera_q,
        &mut visibility_q,
        &mut vertex_3d_q,
        &vertex_2d_q,
        &edge_2d_q,
        &mut meshes,
        &mut materials,
    );
}
