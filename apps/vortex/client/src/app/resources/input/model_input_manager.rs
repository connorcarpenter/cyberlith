use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{Query, Res, SystemState},
    world::{World, Mut},
};

use input::{InputAction, Key, MouseButton};
use math::Vec2;
use render_api::components::Transform;

use vortex_proto::components::{FileExtension, ShapeName, VertexRoot};

use crate::app::{
    components::{OwnedByFileLocal, Vertex2d, Edge2dLocal},
    resources::{file_manager::FileManager, model_manager::ModelManager,
        canvas::Canvas, edge_manager::EdgeManager, input::InputManager, shape_data::CanvasShape,
        tab_manager::TabManager, action::model::ModelAction
    },
};

pub struct ModelInputManager;

impl ModelInputManager {
    pub fn update_input(
        world: &mut World,
        input_manager: &mut InputManager,
        input_actions: Vec<InputAction>,
    ) {
        for action in input_actions {
            match action {
                InputAction::MouseClick(click_type, mouse_position) => {
                    Self::handle_mouse_click(world, input_manager, &mouse_position, click_type)
                }
                InputAction::MouseDragged(click_type, mouse_position, delta) => {
                    Self::handle_mouse_drag(world, input_manager, mouse_position, delta, click_type)
                }
                InputAction::MiddleMouseScroll(scroll_y) => {
                    InputManager::handle_mouse_scroll_wheel(world, scroll_y)
                }
                InputAction::MouseMoved => {
                    input_manager.queue_resync_hover_ui();
                    input_manager.queue_resync_selection_ui();
                }
                InputAction::MouseRelease(MouseButton::Left) => {
                    // input_manager.reset_last_dragged_vertex(world);
                    // Self::reset_last_dragged_edge(world, input_manager);
                }
                InputAction::KeyPress(key) => match key {
                    Key::S
                    | Key::W
                    | Key::D
                    | Key::T
                    | Key::F
                    | Key::Num1
                    | Key::Num2
                    | Key::Num3
                    | Key::Num4
                    | Key::Num5
                    | Key::PageUp
                    | Key::PageDown => InputManager::handle_keypress_camera_controls(world, key),
                    Key::Delete => Self::handle_delete_key_press(world, input_manager),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub(crate) fn handle_delete_key_press(_world: &mut World, _input_manager: &mut InputManager) {
        // todo
    }

    pub(crate) fn handle_mouse_click(
        world: &mut World,
        input_manager: &mut InputManager,
        mouse_position: &Vec2,
        click_type: MouseButton,
    ) {
        // check if mouse position is outside of canvas
        if !world
            .get_resource::<Canvas>()
            .unwrap()
            .is_position_inside(*mouse_position)
        {
            return;
        }

        let selected_shape = input_manager.selected_shape.map(|(_, shape)| shape);
        let hovered_shape = input_manager.hovered_entity.map(|(_, shape)| shape);

        // click_type, selected_shape, hovered_shape
        match (click_type, selected_shape, hovered_shape) {
            (
                MouseButton::Left,
                Some(_),
                Some(CanvasShape::Vertex | CanvasShape::RootVertex | CanvasShape::Face) | None,
            ) | (MouseButton::Right, _, _) => {
                // deselect vertex
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_model_action(
                        world,
                        input_manager,
                        ModelAction::SelectShape(None),
                    );
                });
            }
            (MouseButton::Left, _, Some(CanvasShape::Edge)) => {
                // select hovered shape (or None if there is no hovered shape)
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_model_action(
                        world,
                        input_manager,
                        ModelAction::SelectShape(input_manager.hovered_entity),
                    );
                });
            }
            _ => {}
        }
    }

    pub(crate) fn handle_mouse_drag(
        world: &mut World,
        input_manager: &mut InputManager,
        _mouse_position: Vec2,
        delta: Vec2,
        click_type: MouseButton,
    ) {
        if !world.get_resource::<TabManager>().unwrap().has_focus() {
            return;
        }

        match (click_type, input_manager.selected_shape) {
            (_, _) => InputManager::handle_drag_empty_space(world, click_type, delta),
        }
    }

    pub(crate) fn sync_mouse_hover_ui(
        world: &mut World,
        current_file_entity: &Entity,
        camera_3d_scale: f32,
        mouse_position: &Vec2,
    ) -> Option<(Entity, CanvasShape)> {
        let mut system_state: SystemState<(
            Res<FileManager>,
            Res<EdgeManager>,
            Res<ModelManager>,
            Query<&Transform>,
            Query<&ShapeName>,
            Query<&VertexRoot>,
            Query<(Entity, &OwnedByFileLocal), With<Edge2dLocal>>
        )> = SystemState::new(world);
        let (
            file_manager,
            edge_manager,
            model_manager,
            transform_q,
            shape_name_q,
            vertex_root_q,
            edge_2d_q,
        ) = system_state.get_mut(world);
        let Some(skel_file_entity) = file_manager.file_get_dependency(current_file_entity, FileExtension::Skel) else {
            return None;
        };

        let mut least_distance = f32::MAX;
        let mut least_entity = None;
        let mut is_hovering = false;

        let mtc_2d_entites = model_manager.model_transform_2d_vertices(current_file_entity);

        Self::handle_vertex_hover(
            &transform_q,
            mtc_2d_entites,
            &vertex_root_q,
            camera_3d_scale,
            mouse_position,
            &mut least_distance,
            &mut least_entity,
            &mut is_hovering,
        );

        let mut skel_edge_2d_entities = Vec::new();
        for (edge_2d_entity, owned_by_local) in edge_2d_q.iter() {
            if owned_by_local.file_entity == skel_file_entity {
                skel_edge_2d_entities.push(edge_2d_entity);
            }
        }

        Self::handle_edge_hover(
            &transform_q,
            skel_edge_2d_entities,
            &edge_manager,
            &shape_name_q,
            camera_3d_scale,
            mouse_position,
            &mut least_distance,
            &mut least_entity,
            &mut is_hovering,
        );

        if is_hovering {
            least_entity
        } else {
            None
        }
    }

    fn handle_vertex_hover(
        transform_q: &Query<&Transform>,
        vertex_2d_entities: Vec<Entity>,
        vertex_root_q: &Query<&VertexRoot>,
        camera_3d_scale: f32,
        mouse_position: &Vec2,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        is_hovering: &mut bool,
    ) {
        // check for vertices
        for vertex_2d_entity in vertex_2d_entities {
            let root_opt = vertex_root_q.get(vertex_2d_entity).ok();
            InputManager::hover_check_vertex(transform_q, None, mouse_position, least_distance, least_entity, &vertex_2d_entity, root_opt);
        }

        *is_hovering = *least_distance <= (Vertex2d::DETECT_RADIUS * camera_3d_scale);
    }

    fn handle_edge_hover(
        transform_q: &Query<&Transform>,
        edge_2d_entities: Vec<Entity>,
        edge_manager: &EdgeManager,
        shape_name_q: &Query<&ShapeName>,
        camera_3d_scale: f32,
        mouse_position: &Vec2,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        is_hovering: &mut bool,
    ) {
        // check for edges
        if !*is_hovering {
            for edge_2d_entity in edge_2d_entities {
                InputManager::hover_check_edge(transform_q, Some((edge_manager, shape_name_q)), mouse_position, least_distance, least_entity, &edge_2d_entity);
            }

            *is_hovering = *least_distance <= (Edge2dLocal::DETECT_THICKNESS * camera_3d_scale);
        }
    }
}
