use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res, ResMut},
};

use naia_bevy_client::Client;
use input::Input;
use render_api::components::{Camera, Projection, Transform, Visibility};
use vortex_proto::components::{OwnedByTab, Vertex3d, VertexRootChild};

use crate::app::{
    components::{Compass, Edge2d, Vertex2d},
    resources::{camera_manager::CameraManager, canvas::Canvas, tab_manager::TabManager, action_stack::ActionStack, canvas_manager::CanvasManager},
};

pub fn input(
    mut commands: Commands,
    mut client: Client,
    mut camera_manager: ResMut<CameraManager>,
    canvas: Res<Canvas>,
    mut canvas_manager: ResMut<CanvasManager>,
    mut input: ResMut<Input>,
    mut action_stack: ResMut<ActionStack>,
    tab_manager: Res<TabManager>,
    mut transform_q: Query<&mut Transform>,
    mut camera_q: Query<(&mut Camera, &mut Projection)>,
    mut visibility_q: Query<&mut Visibility>,
    owned_by_tab_q: Query<&OwnedByTab>,
    mut vertex_3d_q: Query<&mut Vertex3d>,
    vertex_2d_q: Query<(Entity, Option<&VertexRootChild>), (With<Vertex2d>, Without<Compass>)>,
    edge_2d_q: Query<(Entity, &Edge2d), Without<Compass>>,
) {
    if !canvas.is_visible() {
        return;
    }
    canvas_manager.update_input(
        &mut commands,
        &mut client,
        &mut camera_manager,
        &mut input,
        &mut action_stack,
        tab_manager.current_tab_id(),
        &mut transform_q,
        &mut camera_q,
        &mut visibility_q,
        &owned_by_tab_q,
        &mut vertex_3d_q,
        &vertex_2d_q,
        &edge_2d_q,
    );
}
