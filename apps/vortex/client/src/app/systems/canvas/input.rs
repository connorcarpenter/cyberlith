use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res, ResMut},
};

use input::Input;
use naia_bevy_client::Client;
use render_api::components::{Camera, Projection, Transform, Visibility};
use vortex_proto::components::{OwnedByFile, Vertex3d, VertexRoot};

use crate::app::{
    components::{Compass, Edge2dLocal, Vertex2d},
    resources::{
        action_stack::ActionStack, camera_manager::CameraManager, canvas::Canvas,
        input_manager::InputManager, shape_manager::ShapeManager, tab_manager::TabManager,
    },
};
use crate::app::components::OwnedByFileLocal;

pub fn input(
    mut commands: Commands,
    mut client: Client,
    mut camera_manager: ResMut<CameraManager>,
    canvas: Res<Canvas>,
    mut shape_manager: ResMut<ShapeManager>,
    mut input_manager: ResMut<InputManager>,
    mut input: ResMut<Input>,
    mut action_stack: ResMut<ActionStack>,
    mut transform_q: Query<&mut Transform>,
    mut camera_q: Query<(&mut Camera, &mut Projection)>,
    mut vertex_3d_q: Query<&mut Vertex3d>,
) {
    if !canvas.is_visible() {
        return;
    }
    let input_actions = input_manager.update_input(&mut input);
    if !input_actions.is_empty() {
        shape_manager.update_input(
            input_actions,
            &mut commands,
            &mut client,
            &mut camera_manager,
            &mut action_stack,
            &mut transform_q,
            &mut camera_q,
            &mut vertex_3d_q,
        );
    }
}

pub fn update_mouse_hover(
    tab_manager: Res<TabManager>,
    canvas: Res<Canvas>,
    camera_manager: Res<CameraManager>,
    input: Res<Input>,
    mut shape_manager: ResMut<ShapeManager>,
    mut transform_q: Query<(&mut Transform, Option<&Compass>)>,
    vertex_2d_q: Query<(Entity, Option<&VertexRoot>), (With<Vertex2d>, Without<Compass>)>,
    edge_2d_q: Query<(Entity, &Edge2dLocal), Without<Compass>>,
    owned_by_tab_q: Query<&OwnedByFileLocal>,
    mut visibility_q: Query<&mut Visibility>,
) {
    if !canvas.is_visible() {
        return;
    }
    shape_manager.update_mouse_hover(
        &camera_manager,
        tab_manager.current_tab_entity(),
        input.mouse_position(),
        &mut transform_q,
        &mut visibility_q,
        &owned_by_tab_q,
        &vertex_2d_q,
        &edge_2d_q,
    );
}
