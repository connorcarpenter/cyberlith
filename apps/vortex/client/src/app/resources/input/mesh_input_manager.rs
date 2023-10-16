use bevy_ecs::{
    system::{Commands, Query, Res, ResMut, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt};

use input::{InputAction, Key, MouseButton};
use math::{convert_2d_to_3d, Vec2, Vec3};
use render_api::components::{Camera, CameraProjection, Projection, Transform};

use vortex_proto::components::Vertex3d;

use crate::app::{
    components::VertexTypeData,
    resources::{
        action::shape::ShapeAction, camera_manager::CameraManager, canvas::Canvas,
        edge_manager::EdgeManager, face_manager::FaceManager, input::InputManager,
        shape_data::CanvasShape, tab_manager::TabManager, vertex_manager::VertexManager,
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

        let mut system_state: SystemState<(
            Res<CameraManager>,
            Res<VertexManager>,
            Res<EdgeManager>,
            Query<(&Camera, &Projection)>,
            Query<&Transform>,
        )> = SystemState::new(world);
        let (camera_manager, vertex_manager, edge_manager, camera_q, transform_q) =
            system_state.get_mut(world);

        let selected_shape = input_manager.selected_shape.map(|(_, shape)| shape);
        let hovered_shape = input_manager.hovered_entity.map(|(_, shape)| shape);

        // click_type, selected_shape, hovered_shape, current_file_type
        match (click_type, selected_shape, hovered_shape) {
            (MouseButton::Left, Some(CanvasShape::Edge | CanvasShape::Face), _) => {
                // should not ever be able to attach something to an edge or face?
                // select hovered entity
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::SelectShape(input_manager.hovered_entity),
                    );
                });
                return;
            }
            (MouseButton::Left, Some(_), Some(CanvasShape::Edge | CanvasShape::Face)) => {
                // should not ever be able to attach something to an edge or face?
                // select hovered entity
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::SelectShape(input_manager.hovered_entity),
                    );
                });
                return;
            }
            (
                MouseButton::Left,
                Some(CanvasShape::Vertex | CanvasShape::RootVertex),
                Some(CanvasShape::Vertex | CanvasShape::RootVertex),
            ) => {
                // link vertices together
                let (vertex_2d_entity_a, _) = input_manager.selected_shape.unwrap();
                let (vertex_2d_entity_b, _) = input_manager.hovered_entity.unwrap();
                if vertex_2d_entity_a == vertex_2d_entity_b {
                    return;
                }

                // check if edge already exists
                if edge_manager
                    .edge_2d_entity_from_vertices(
                        &vertex_manager,
                        vertex_2d_entity_a,
                        vertex_2d_entity_b,
                    )
                    .is_some()
                {
                    // select vertex
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_shape_action(
                            world,
                            input_manager,
                            ShapeAction::SelectShape(Some((
                                vertex_2d_entity_b,
                                CanvasShape::Vertex,
                            ))),
                        );
                    });
                    return;
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
                    return;
                }
            }
            (MouseButton::Left, Some(CanvasShape::Vertex | CanvasShape::RootVertex), None) => {
                // create new vertex

                // get camera
                let camera_3d = camera_manager.camera_3d_entity().unwrap();
                let camera_transform: Transform = *transform_q.get(camera_3d).unwrap();
                let (camera, camera_projection) = camera_q.get(camera_3d).unwrap();

                let camera_viewport = camera.viewport.unwrap();
                let view_matrix = camera_transform.view_matrix();
                let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

                // get 2d vertex transform
                let (vertex_2d_entity, _) = input_manager.selected_shape.unwrap();
                let Ok(vertex_2d_transform) = transform_q.get(vertex_2d_entity) else {
                    warn!(
                        "Selected vertex entity: {:?} has no Transform",
                        vertex_2d_entity
                    );
                    return;
                };
                // convert 2d to 3d
                let new_3d_position = convert_2d_to_3d(
                    &view_matrix,
                    &projection_matrix,
                    &camera_viewport.size_vec2(),
                    &mouse_position,
                    vertex_2d_transform.translation.z,
                );

                // spawn new vertex
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::CreateVertex(
                            VertexTypeData::Mesh(vec![(vertex_2d_entity, None)], Vec::new()),
                            new_3d_position,
                            None,
                        ),
                    );
                });
            }
            (
                MouseButton::Left,
                None,
                Some(CanvasShape::RootVertex | CanvasShape::Vertex | CanvasShape::Edge),
            ) => {
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::SelectShape(input_manager.hovered_entity),
                    );
                });
            }
            (MouseButton::Left, None, Some(CanvasShape::Face)) => {
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_shape_action(
                        world,
                        input_manager,
                        ShapeAction::SelectShape(input_manager.hovered_entity),
                    );
                });
            }
            (MouseButton::Right, _, _) => {
                // deselect vertex
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

        let shape_is_selected = input_manager.selected_shape.is_some();
        let shape_can_drag = shape_is_selected
            && match input_manager.selected_shape.unwrap().1 {
                CanvasShape::Vertex => true,
                CanvasShape::Edge => false,
                _ => false,
            };

        if shape_is_selected && shape_can_drag {
            match click_type {
                MouseButton::Left => {
                    match input_manager.selected_shape.unwrap() {
                        (vertex_2d_entity, CanvasShape::Vertex) => {
                            // move vertex
                            let Some(vertex_3d_entity) = world.get_resource::<VertexManager>().unwrap().vertex_entity_2d_to_3d(&vertex_2d_entity) else {
                                warn!(
                                    "Selected vertex entity: {:?} has no 3d counterpart",
                                    vertex_2d_entity
                                );
                                return;
                            };

                            let mut system_state: SystemState<(
                                Commands,
                                Client,
                                Res<CameraManager>,
                                ResMut<VertexManager>,
                                ResMut<Canvas>,
                                Query<(&Camera, &Projection)>,
                                Query<&Transform>,
                                Query<&mut Vertex3d>,
                            )> = SystemState::new(world);
                            let (
                                mut commands,
                                client,
                                camera_manager,
                                mut vertex_manager,
                                mut canvas,
                                camera_q,
                                transform_q,
                                mut vertex_3d_q,
                            ) = system_state.get_mut(world);

                            // check status
                            let auth_status = commands
                                .entity(vertex_3d_entity)
                                .authority(&client)
                                .unwrap();
                            if !(auth_status.is_requested() || auth_status.is_granted()) {
                                // only continue to mutate if requested or granted authority over vertex
                                info!("No authority over vertex, skipping..");
                                return;
                            }

                            // get camera
                            let camera_3d = camera_manager.camera_3d_entity().unwrap();
                            let camera_transform: Transform = *transform_q.get(camera_3d).unwrap();
                            let (camera, camera_projection) = camera_q.get(camera_3d).unwrap();

                            let camera_viewport = camera.viewport.unwrap();
                            let view_matrix = camera_transform.view_matrix();
                            let projection_matrix =
                                camera_projection.projection_matrix(&camera_viewport);

                            // get 2d vertex transform
                            let vertex_2d_transform = transform_q.get(vertex_2d_entity).unwrap();

                            // convert 2d to 3d
                            let new_3d_position = convert_2d_to_3d(
                                &view_matrix,
                                &projection_matrix,
                                &camera_viewport.size_vec2(),
                                &mouse_position,
                                vertex_2d_transform.translation.z,
                            );

                            // set networked 3d vertex position
                            let mut vertex_3d = vertex_3d_q.get_mut(vertex_3d_entity).unwrap();

                            vertex_manager.update_last_vertex_dragged(
                                vertex_2d_entity,
                                vertex_3d.as_vec3(),
                                new_3d_position,
                            );

                            vertex_3d.set_vec3(&new_3d_position);

                            // redraw
                            canvas.queue_resync_shapes();
                        }
                        _ => {
                            panic!("Shouldn't be possible");
                        }
                    }
                }
                MouseButton::Right => {
                    // TODO: dunno if this is possible? shouldn't the vertex be deselected?
                }
                _ => {}
            }
        } else {
            InputManager::handle_drag_empty_space(world, click_type, delta);
        }
    }
}
