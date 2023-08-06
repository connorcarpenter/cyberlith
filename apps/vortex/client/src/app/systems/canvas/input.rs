use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res, ResMut},
};

use input::Input;
use naia_bevy_client::Client;
use render_api::components::{Camera, Projection, Transform, Visibility};
use vortex_proto::components::{OwnedByTab, Vertex3d, VertexRootChild};

use crate::app::{
    components::{Compass, Edge2dLocal, Vertex2d},
    resources::{
        action_stack::ActionStack, camera_manager::CameraManager, canvas::Canvas,
        input_manager::InputManager, tab_manager::TabManager, vertex_manager::VertexManager,
    },
};

pub fn input(
    mut commands: Commands,
    mut client: Client,
    mut camera_manager: ResMut<CameraManager>,
    canvas: Res<Canvas>,
    mut vertex_manager: ResMut<VertexManager>,
    mut input_manager: ResMut<InputManager>,
    mut input: ResMut<Input>,
    mut action_stack: ResMut<ActionStack>,
    tab_manager: Res<TabManager>,
    mut transform_q: Query<&mut Transform>,
    mut camera_q: Query<(&mut Camera, &mut Projection)>,
    mut visibility_q: Query<&mut Visibility>,
    owned_by_tab_q: Query<&OwnedByTab>,
    mut vertex_3d_q: Query<&mut Vertex3d>,
    vertex_2d_q: Query<(Entity, Option<&VertexRootChild>), (With<Vertex2d>, Without<Compass>)>,
    edge_2d_q: Query<(Entity, &Edge2dLocal), Without<Compass>>,
) {
    if !canvas.is_visible() {
        return;
    }
    let input_actions = input_manager.update_input(&mut input);
    if !input_actions.is_empty() {
        vertex_manager.update_input(
            input_actions,
            tab_manager.current_tab_id(),
            input.mouse_position(),
            &mut commands,
            &mut client,
            &mut camera_manager,
            &mut action_stack,
            &mut transform_q,
            &mut camera_q,
            &mut visibility_q,
            &owned_by_tab_q,
            &mut vertex_3d_q,
            &vertex_2d_q,
            &edge_2d_q,
        );
    }
}
