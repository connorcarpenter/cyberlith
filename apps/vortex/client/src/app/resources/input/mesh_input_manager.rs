use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res, ResMut, SystemState},
    world::{Mut, World},
};
use bevy_log::warn;

use naia_bevy_client::{Client, CommandsExt};

use input::{InputAction, Key, MouseButton};
use math::{Vec2, Vec3};
use render_api::{
    components::{Transform, Visibility},
    shapes::{distance_to_2d_line, get_2d_line_transform_endpoint},
};

use vortex_proto::components::VertexRoot;

use crate::app::{
    components::{Edge2dLocal, FaceIcon2d, LocalShape, Vertex2d, VertexTypeData},
    resources::{
        action::shape::ShapeAction, canvas::Canvas, edge_manager::EdgeManager,
        face_manager::FaceManager, input::InputManager, shape_data::CanvasShape,
        tab_manager::TabManager, vertex_manager::VertexManager,
    },
};

pub struct MeshInputManager;

impl MeshInputManager {
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
                    input_manager.reset_last_dragged_vertex(world)
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
                    Key::Delete => Self::handle_delete_key_press(input_manager, world),
                    Key::Insert => Self::handle_insert_key_press(world, input_manager),
                    _ => {}
                },
                _ => {}
            }
        }
    }

    pub(crate) fn handle_insert_key_press(world: &mut World, input_manager: &mut InputManager) {
        if input_manager.selected_shape.is_some() {
            return;
        }
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.current_tab_execute_shape_action(
                world,
                input_manager,
                ShapeAction::CreateVertex(
                    VertexTypeData::Mesh(Vec::new(), Vec::new()),
                    Vec3::ZERO,
                    None,
                ),
            );
        })
    }

    pub(crate) fn handle_delete_key_press(input_manager: &mut InputManager, world: &mut World) {
        match input_manager.selected_shape {
            Some((vertex_2d_entity, CanvasShape::Vertex)) => {
                input_manager.handle_delete_vertex_action(world, &vertex_2d_entity)
            }
            Some((edge_2d_entity, CanvasShape::Edge)) => {
                let mut system_state: SystemState<(Commands, Client, Res<EdgeManager>)> =
                    SystemState::new(world);
                let (mut commands, mut client, edge_manager) = system_state.get_mut(world);

                // delete edge
                let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();

                // check whether we can delete edge
                let auth_status = commands.entity(edge_3d_entity).authority(&client).unwrap();
                if !auth_status.is_granted() && !auth_status.is_available() {
                    // do nothing, edge is not available
                    // TODO: queue for deletion? check before this?
                    warn!("Edge {:?} is not available for deletion!", edge_3d_entity);
                    return;
                }

                let auth_status = commands.entity(edge_3d_entity).authority(&client).unwrap();
                if !auth_status.is_granted() {
                    // request authority if needed
                    commands
                        .entity(edge_3d_entity)
                        .request_authority(&mut client);
                }

                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::DeleteEdge(edge_2d_entity, None),
                    );
                });

                input_manager.selected_shape = None;
            }
            Some((face_2d_entity, CanvasShape::Face)) => {
                let mut system_state: SystemState<(Commands, Client, Res<FaceManager>)> =
                    SystemState::new(world);
                let (mut commands, mut client, face_manager) = system_state.get_mut(world);

                let face_3d_entity = face_manager.face_entity_2d_to_3d(&face_2d_entity).unwrap();

                // check whether we can delete edge
                let auth_status = commands.entity(face_3d_entity).authority(&client).unwrap();
                if !auth_status.is_granted() && !auth_status.is_available() {
                    // do nothing, face is not available
                    // TODO: queue for deletion? check before this?
                    warn!("Face `{:?}` is not available for deletion!", face_3d_entity);
                    return;
                }

                let auth_status = commands.entity(face_3d_entity).authority(&client).unwrap();
                if !auth_status.is_granted() {
                    // request authority if needed
                    commands
                        .entity(face_3d_entity)
                        .request_authority(&mut client);
                }

                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::DeleteFace(face_2d_entity),
                    );
                });

                input_manager.selected_shape = None;
            }
            _ => {}
        }
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
            (MouseButton::Left, Some(CanvasShape::Vertex), Some(CanvasShape::Vertex)) => {
                Self::link_vertices(world, input_manager);
            }
            (MouseButton::Left, Some(CanvasShape::Vertex), None) => {
                // create new vertex
                let (vertex_2d_entity, _) = input_manager.selected_shape.unwrap();
                let vertex_type_data =
                    VertexTypeData::Mesh(vec![(vertex_2d_entity, None)], Vec::new());
                InputManager::handle_create_new_vertex(
                    world,
                    input_manager,
                    &mouse_position,
                    vertex_2d_entity,
                    vertex_type_data,
                );
            }
            (MouseButton::Left, _, _) => {
                // select hovered shape (or None if there is no hovered shape)
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::SelectShape(input_manager.hovered_entity),
                    );
                });
            }
            (MouseButton::Right, _, _) => {
                // deselect shape
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::SelectShape(None),
                    );
                });
            }
            _ => {}
        }
    }

    fn link_vertices(world: &mut World, input_manager: &mut InputManager) {
        let mut system_state: SystemState<(Res<VertexManager>, Res<EdgeManager>)> =
            SystemState::new(world);
        let (vertex_manager, edge_manager) = system_state.get_mut(world);

        // link vertices together
        let (vertex_2d_entity_a, _) = input_manager.selected_shape.unwrap();
        let (vertex_2d_entity_b, _) = input_manager.hovered_entity.unwrap();
        if vertex_2d_entity_a == vertex_2d_entity_b {
            return;
        }

        // check if edge already exists
        if edge_manager
            .edge_2d_entity_from_vertices(&vertex_manager, vertex_2d_entity_a, vertex_2d_entity_b)
            .is_some()
        {
            // select vertex
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_shape_action(
                    world,
                    input_manager,
                    ShapeAction::SelectShape(Some((vertex_2d_entity_b, CanvasShape::Vertex))),
                );
            });
        } else {
            // create edge
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_shape_action(
                    world,
                    input_manager,
                    ShapeAction::CreateEdge(
                        vertex_2d_entity_a,
                        vertex_2d_entity_b,
                        (vertex_2d_entity_b, CanvasShape::Vertex),
                        None,
                        None,
                    ),
                );
            });
        }
    }

    pub(crate) fn handle_mouse_drag(
        world: &mut World,
        input_manager: &mut InputManager,
        mouse_position: Vec2,
        delta: Vec2,
        click_type: MouseButton,
    ) {
        if !world.get_resource::<TabManager>().unwrap().has_focus() {
            return;
        }

        match (click_type, input_manager.selected_shape) {
            (MouseButton::Left, Some((vertex_2d_entity, CanvasShape::Vertex))) => {
                InputManager::handle_vertex_drag(world, &vertex_2d_entity, &mouse_position)
            }
            (_, _) => InputManager::handle_drag_empty_space(world, click_type, delta),
        }
    }

    pub(crate) fn sync_mouse_hover_ui(
        world: &mut World,
        input_manager: &mut InputManager,
        mouse_position: &Vec2,
    ) {
        let mut system_state: SystemState<(
            ResMut<Canvas>,
            Res<TabManager>,
            Query<(&mut Transform, Option<&LocalShape>)>,
            Query<&Visibility>,
            Query<(Entity, Option<&VertexRoot>), (With<Vertex2d>, Without<LocalShape>)>,
            Query<(Entity, &Edge2dLocal), Without<LocalShape>>,
            Query<(Entity, &FaceIcon2d)>,
        )> = SystemState::new(world);
        let (
            mut canvas,
            tab_manager,
            mut transform_q,
            visibility_q,
            vertex_2d_q,
            edge_2d_q,
            face_2d_q,
        ) = system_state.get_mut(world);

        let Some(current_tab_state) = tab_manager.current_tab_state() else {
            return;
        };
        let camera_state = &current_tab_state.camera_state;

        let camera_3d_scale = camera_state.camera_3d_scale();

        let mut least_distance = f32::MAX;
        let mut least_entity = None;
        let mut is_hovering;

        // check for vertices
        for (vertex_2d_entity, root_opt) in vertex_2d_q.iter() {
            let Ok(visibility) = visibility_q.get(vertex_2d_entity) else {
                panic!("Vertex entity has no Visibility");
            };
            if !visibility.visible {
                continue;
            }

            let (vertex_transform, _) = transform_q.get(vertex_2d_entity).unwrap();
            let vertex_position = vertex_transform.translation.truncate();
            let distance = vertex_position.distance(*mouse_position);
            if distance < least_distance {
                least_distance = distance;

                let shape = match root_opt {
                    Some(_) => CanvasShape::RootVertex,
                    None => CanvasShape::Vertex,
                };

                least_entity = Some((vertex_2d_entity, shape));
            }
        }

        is_hovering = least_distance <= (Vertex2d::DETECT_RADIUS * camera_3d_scale);

        // check for edges
        if !is_hovering {
            for (edge_2d_entity, _) in edge_2d_q.iter() {
                // check visibility
                let Ok(visibility) = visibility_q.get(edge_2d_entity) else {
                    panic!("entity has no Visibility");
                };
                if !visibility.visible {
                    continue;
                }

                let (edge_transform, _) = transform_q.get(edge_2d_entity).unwrap();
                let edge_start = edge_transform.translation.truncate();
                let edge_end = get_2d_line_transform_endpoint(&edge_transform);

                let distance = distance_to_2d_line(*mouse_position, edge_start, edge_end);
                if distance < least_distance {
                    least_distance = distance;
                    least_entity = Some((edge_2d_entity, CanvasShape::Edge));
                }
            }

            is_hovering = least_distance <= (Edge2dLocal::DETECT_THICKNESS * camera_3d_scale);
        }

        // check for faces
        if !is_hovering {
            for (face_entity, _) in face_2d_q.iter() {
                // check tab ownership, skip faces from other tabs
                let Ok(visibility) = visibility_q.get(face_entity) else {
                    panic!("entity has no Visibility");
                };
                if !visibility.visible {
                    continue;
                }

                let (face_transform, _) = transform_q.get(face_entity).unwrap();
                let face_position = face_transform.translation.truncate();
                let distance = face_position.distance(*mouse_position);
                if distance < least_distance {
                    least_distance = distance;

                    least_entity = Some((face_entity, CanvasShape::Face));
                }
            }

            is_hovering = least_distance <= (FaceIcon2d::DETECT_RADIUS * camera_3d_scale);
        }

        // define old and new hovered states
        let old_hovered_entity = input_manager.hovered_entity;
        let next_hovered_entity = if is_hovering { least_entity } else { None };

        input_manager.sync_hover_shape_scale(&mut transform_q, camera_3d_scale);

        // hover state did not change
        if old_hovered_entity == next_hovered_entity {
            return;
        }

        // apply
        input_manager.hovered_entity = next_hovered_entity;
        canvas.queue_resync_shapes_light();
    }
}
