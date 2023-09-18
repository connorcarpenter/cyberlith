use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res, ResMut},
};
use bevy_ecs::system::SystemState;
use bevy_ecs::world::{Mut, World};

use naia_bevy_client::Client;

use input::Input;
use render_api::components::{Camera, Projection, Transform, Visibility};

use vortex_proto::components::{EdgeAngle, ShapeName, Vertex3d, VertexRoot};

use crate::app::{
    components::{Edge2dLocal, FaceIcon2d, LocalShape, Vertex2d},
    resources::{
        animation_manager::AnimationManager, camera_manager::CameraManager, canvas::Canvas,
        edge_manager::EdgeManager, face_manager::FaceManager, file_manager::FileManager,
        input_manager::InputManager, tab_manager::TabManager, vertex_manager::VertexManager,
    },
};

pub fn input(world: &mut World) {
    let mut system_state: SystemState<(
        Res<Canvas>,
        ResMut<Input>,
    )> = SystemState::new(world);
    let (canvas, mut input) = system_state.get_mut(world);

    if !canvas.is_visible() {
        return;
    }

    let input_actions = input.take_actions();

    world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
        input_manager.update_input(
            input_actions,
            world,
        );
    });
}

pub fn update_mouse_hover(
    mut canvas: ResMut<Canvas>,
    input: Res<Input>,
    tab_manager: Res<TabManager>,
    mut input_manager: ResMut<InputManager>,
    mut transform_q: Query<(&mut Transform, Option<&LocalShape>)>,
    visibility_q: Query<&Visibility>,
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
    let current_tab_camera_state = &current_tab_state.camera_state;

    input_manager.sync_mouse_hover_ui(
        &mut canvas,
        input.mouse_position(),
        current_tab_camera_state,
        &mut transform_q,
        &visibility_q,
        &vertex_2d_q,
        &edge_2d_q,
        &face_2d_q,
    );
}
