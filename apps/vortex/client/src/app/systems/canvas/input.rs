use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Query, Res, ResMut, SystemState},
    world::{Mut, World},
};

use input::Input;
use render_api::components::{Transform, Visibility};

use vortex_proto::components::{FileExtension, ShapeName, VertexRoot};

use crate::app::{
    components::{Edge2dLocal, FaceIcon2d, LocalShape, Vertex2d},
    resources::{
        animation_manager::AnimationManager, canvas::Canvas, edge_manager::EdgeManager,
        file_manager::FileManager, input::InputManager, tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
};

pub fn input(world: &mut World) {
    let mut system_state: SystemState<(Res<Canvas>, ResMut<Input>)> = SystemState::new(world);
    let (canvas, mut input) = system_state.get_mut(world);

    if !canvas.is_visible() {
        return;
    }

    let input_actions = input.take_actions();

    world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
        input_manager.update_input(input_actions, world);
    });
}

pub fn update_mouse_hover(
    mut canvas: ResMut<Canvas>,
    input: Res<Input>,
    file_manager: Res<FileManager>,
    tab_manager: Res<TabManager>,
    mut input_manager: ResMut<InputManager>,
    vertex_manager: Res<VertexManager>,
    edge_manager: Res<EdgeManager>,
    mut animation_manager: ResMut<AnimationManager>,
    mut transform_q: Query<(&mut Transform, Option<&LocalShape>)>,
    visibility_q: Query<&Visibility>,
    shape_name_q: Query<&ShapeName>,
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
    let Some(current_tab_entity) = tab_manager.current_tab_entity() else {
        return;
    };

    let file_type = file_manager.get_file_type(current_tab_entity);

    let current_tab_camera_state = &current_tab_state.camera_state;

    if file_type == FileExtension::Anim {
        let canvas_size = canvas.canvas_texture_size();
        if animation_manager.is_framing() {
            animation_manager.sync_mouse_hover_ui_framing(
                current_tab_entity,
                canvas_size,
                input.mouse_position(),
            );
            return;
        }
    }

    input_manager.sync_mouse_hover_ui(
        file_type,
        &mut canvas,
        &vertex_manager,
        &edge_manager,
        &animation_manager,
        &mut transform_q,
        &visibility_q,
        &shape_name_q,
        &vertex_2d_q,
        &edge_2d_q,
        &face_2d_q,
        current_tab_camera_state,
        input.mouse_position(),
    );
}
