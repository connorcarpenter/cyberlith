use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res, ResMut},
};

use naia_bevy_client::Client;

use input::Input;
use render_api::components::{Camera, Projection, Transform};

use vortex_proto::components::{EdgeAngle, Vertex3d, VertexRoot};

use crate::app::{
    components::{Edge2dLocal, FaceIcon2d, LocalShape, OwnedByFileLocal, Vertex2d},
    resources::{
        animation_manager::AnimationManager, camera_manager::CameraManager, canvas::Canvas,
        edge_manager::EdgeManager, face_manager::FaceManager, input_manager::InputManager,
        tab_manager::TabManager, vertex_manager::VertexManager,
    },
};

pub fn input(
    mut commands: Commands,
    mut client: Client,
    mut camera_manager: ResMut<CameraManager>,
    mut canvas: ResMut<Canvas>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    face_manager: Res<FaceManager>,
    mut animation_manager: ResMut<AnimationManager>,
    mut tab_manager: ResMut<TabManager>,
    mut input_manager: ResMut<InputManager>,
    mut input: ResMut<Input>,
    mut transform_q: Query<&mut Transform>,
    mut camera_q: Query<(&mut Camera, &mut Projection)>,
    mut vertex_3d_q: Query<&mut Vertex3d>,
    mut edge_angle_q: Query<&mut EdgeAngle>,
) {
    if !canvas.is_visible() {
        return;
    }
    let Some(current_tab_state) = tab_manager.current_tab_state_mut() else {
        return;
    };
    let input_actions = input.take_actions();
    input_manager.update_input(
        input_actions,
        &mut commands,
        &mut client,
        &mut canvas,
        &mut camera_manager,
        &mut animation_manager,
        current_tab_state,
        &mut vertex_manager,
        &mut edge_manager,
        &face_manager,
        &mut transform_q,
        &mut camera_q,
        &mut vertex_3d_q,
        &mut edge_angle_q,
    );
}

pub fn update_mouse_hover(
    mut canvas: ResMut<Canvas>,
    input: Res<Input>,
    tab_manager: ResMut<TabManager>,
    mut input_manager: ResMut<InputManager>,
    mut transform_q: Query<(&mut Transform, Option<&LocalShape>)>,
    owned_by_tab_q: Query<&OwnedByFileLocal>,
    vertex_2d_q: Query<(Entity, Option<&VertexRoot>), (With<Vertex2d>, Without<LocalShape>)>,
    edge_2d_q: Query<(Entity, &Edge2dLocal), Without<LocalShape>>,
    face_2d_q: Query<(Entity, &FaceIcon2d)>,
) {
    if !canvas.is_visible() {
        return;
    }

    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };

    if let Some(current_tab_entity) = tab_manager.current_tab_entity() {
        let current_tab_camera_state = &current_tab_state.camera_state;

        input_manager.sync_mouse_hover_ui(
            &mut canvas,
            *current_tab_entity,
            input.mouse_position(),
            current_tab_camera_state,
            &mut transform_q,
            &owned_by_tab_q,
            &vertex_2d_q,
            &edge_2d_q,
            &face_2d_q,
        );
    }
}
