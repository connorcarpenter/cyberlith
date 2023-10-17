use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Commands, Query, Res, ResMut, SystemState},
    world::{Mut, World},
};
use bevy_log::warn;

use naia_bevy_client::{Client, CommandsExt, Instant};

use input::{InputAction, Key, MouseButton};
use math::Vec2;
use render_api::{
    components::{Transform, Visibility},
    shapes::{distance_to_2d_line, get_2d_line_transform_endpoint},
};

use vortex_proto::components::{AnimRotation, ShapeName, VertexRoot};

use crate::app::{
    components::{Edge2dLocal, LocalShape, Vertex2d},
    resources::{
        action::animation::AnimAction,
        animation_manager::AnimationManager,
        canvas::Canvas,
        edge_manager::EdgeManager,
        input::{CardinalDirection, InputManager},
        shape_data::CanvasShape,
        tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
};

pub struct AnimInputManager;

impl AnimInputManager {
    pub fn update_input(
        world: &mut World,
        input_manager: &mut InputManager,
        input_actions: Vec<InputAction>,
    ) {
        let animation_manager = world.get_resource_mut::<AnimationManager>().unwrap();
        if animation_manager.is_posing() {
            Self::update_input_posing(world, input_manager, input_actions);
        } else {
            Self::update_input_framing(world, input_manager, input_actions);
        }
    }

    fn update_input_framing(
        world: &mut World,
        input_manager: &mut InputManager,
        input_actions: Vec<InputAction>,
    ) {
        for action in input_actions {
            match action {
                InputAction::MouseClick(click_type, mouse_position) => {
                    Self::handle_mouse_click_framing(
                        world,
                        input_manager,
                        click_type,
                        &mouse_position,
                    )
                }
                InputAction::MouseDragged(click_type, _mouse_position, delta) => {
                    Self::handle_mouse_drag_framing(world, click_type, delta)
                }
                InputAction::MiddleMouseScroll(scroll_y) => {
                    Self::handle_mouse_scroll_framing(world, scroll_y)
                }
                InputAction::MouseMoved => {
                    input_manager.queue_resync_hover_ui();
                }
                InputAction::KeyPress(key) => match key {
                    Key::Delete => Self::handle_delete_frame(world, input_manager),
                    Key::Insert => Self::handle_insert_frame(world, input_manager),
                    Key::Space => Self::handle_play_pause(world),
                    Key::Enter => {
                        let mut system_state: SystemState<(
                            ResMut<Canvas>,
                            ResMut<AnimationManager>,
                        )> = SystemState::new(world);
                        let (mut canvas, mut animation_manager) = system_state.get_mut(world);
                        animation_manager.set_posing(&mut canvas);
                    }
                    Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
                        let dir = match key {
                            Key::ArrowLeft => CardinalDirection::West,
                            Key::ArrowRight => CardinalDirection::East,
                            Key::ArrowUp => CardinalDirection::North,
                            Key::ArrowDown => CardinalDirection::South,
                            _ => panic!("Unexpected key: {:?}", key),
                        };
                        let current_file_entity = *world
                            .get_resource::<TabManager>()
                            .unwrap()
                            .current_tab_entity()
                            .unwrap();
                        let mut animation_manager =
                            world.get_resource_mut::<AnimationManager>().unwrap();
                        if let Some((prev_index, next_index)) =
                            animation_manager.framing_navigate(&current_file_entity, dir)
                        {
                            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                                tab_manager.current_tab_execute_anim_action(
                                    world,
                                    input_manager,
                                    AnimAction::SelectFrame(
                                        current_file_entity,
                                        next_index,
                                        prev_index,
                                    ),
                                );
                            });
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn update_input_posing(
        world: &mut World,
        input_manager: &mut InputManager,
        input_actions: Vec<InputAction>,
    ) {
        for action in input_actions {
            match action {
                InputAction::MouseClick(click_type, mouse_position) => {
                    Self::handle_mouse_click_posing(
                        world,
                        input_manager,
                        click_type,
                        &mouse_position,
                    )
                }
                InputAction::MouseDragged(click_type, mouse_position, delta) => {
                    Self::handle_mouse_drag_posing(
                        world,
                        input_manager,
                        mouse_position,
                        click_type,
                        delta,
                    )
                }
                InputAction::MiddleMouseScroll(scroll_y) => {
                    InputManager::handle_mouse_scroll_wheel(world, scroll_y)
                }
                InputAction::MouseMoved => {
                    input_manager.queue_resync_hover_ui();
                    input_manager.queue_resync_selection_ui();
                }
                InputAction::MouseRelease(MouseButton::Left) => {
                    Self::reset_last_dragged_rotation(input_manager, world)
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
                    Key::E => InputManager::handle_edge_angle_visibility_toggle(world),
                    Key::Space => {
                        if world
                            .get_resource::<AnimationManager>()
                            .unwrap()
                            .preview_frame_selected()
                        {
                            Self::handle_play_pause(world);
                        }
                    }
                    Key::Escape => {
                        let mut animation_manager =
                            world.get_resource_mut::<AnimationManager>().unwrap();
                        animation_manager.set_framing();
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }

    fn handle_mouse_click_framing(
        world: &mut World,
        input_manager: &mut InputManager,
        click_type: MouseButton,
        mouse_position: &Vec2,
    ) {
        if click_type != MouseButton::Left {
            return;
        }

        // check if mouse position is outside of canvas
        if !world
            .get_resource::<Canvas>()
            .unwrap()
            .is_position_inside(*mouse_position)
        {
            return;
        }

        let animation_manager = world.get_resource::<AnimationManager>().unwrap();

        let current_frame_index = animation_manager.current_frame_index();
        let frame_index_hover = animation_manager.frame_index_hover();

        if frame_index_hover.is_some() {
            let frame_index_hover = frame_index_hover.unwrap();

            let double_clicked = frame_index_hover == input_manager.last_frame_index_hover
                && input_manager.last_left_click_instant.elapsed().as_millis() < 500;
            input_manager.last_left_click_instant = Instant::now();
            input_manager.last_frame_index_hover = frame_index_hover;

            if frame_index_hover == 0 {
                // clicked preview frame
                if double_clicked {
                    let mut system_state: SystemState<(ResMut<Canvas>, ResMut<AnimationManager>)> =
                        SystemState::new(world);
                    let (mut canvas, mut animation_manager) = system_state.get_mut(world);
                    animation_manager.set_posing(&mut canvas);
                    animation_manager.set_preview_frame_selected();
                }
            } else {
                if current_frame_index != frame_index_hover - 1 {
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        let current_file_entity = *tab_manager.current_tab_entity().unwrap();
                        tab_manager.current_tab_execute_anim_action(
                            world,
                            input_manager,
                            AnimAction::SelectFrame(
                                current_file_entity,
                                frame_index_hover - 1,
                                current_frame_index,
                            ),
                        );
                    });
                }

                if double_clicked {
                    let mut system_state: SystemState<(ResMut<Canvas>, ResMut<AnimationManager>)> =
                        SystemState::new(world);
                    let (mut canvas, mut animation_manager) = system_state.get_mut(world);
                    animation_manager.set_posing(&mut canvas);
                }
            }
        }
    }

    fn handle_mouse_click_posing(
        world: &mut World,
        input_manager: &mut InputManager,
        click_type: MouseButton,
        mouse_position: &Vec2,
    ) {
        // check if mouse position is outside of canvas
        if !world
            .get_resource::<Canvas>()
            .unwrap()
            .is_position_inside(*mouse_position)
        {
            return;
        }

        if world
            .get_resource::<AnimationManager>()
            .unwrap()
            .preview_frame_selected()
        {
            return;
        }

        let selected_shape = input_manager.selected_shape.map(|(_, shape)| shape);
        let hovered_shape = input_manager.hovered_entity.map(|(_, shape)| shape);

        // click_type, selected_shape, hovered_shape
        match (click_type, selected_shape, hovered_shape) {
            (MouseButton::Left, _, Some(CanvasShape::Vertex | CanvasShape::Edge) | None) => {
                // select hovered entity
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_anim_action(
                        world,
                        input_manager,
                        AnimAction::SelectShape(input_manager.hovered_entity),
                    );
                });
            }
            (MouseButton::Right, _, _) => {
                // deselect vertex
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_anim_action(
                        world,
                        input_manager,
                        AnimAction::SelectShape(None),
                    );
                });
            }
            _ => {}
        }
    }

    fn handle_mouse_drag_framing(world: &mut World, click_type: MouseButton, delta: Vec2) {
        if !world.get_resource::<TabManager>().unwrap().has_focus() {
            return;
        }

        if click_type != MouseButton::Left {
            return;
        }

        world
            .get_resource_mut::<AnimationManager>()
            .unwrap()
            .handle_mouse_drag_anim_framing(delta.y);
    }

    fn handle_mouse_drag_posing(
        world: &mut World,
        input_manager: &mut InputManager,
        mouse_position: Vec2,
        click_type: MouseButton,
        delta: Vec2,
    ) {
        if !world.get_resource::<TabManager>().unwrap().has_focus() {
            return;
        }

        let current_file_entity = *world
            .get_resource::<TabManager>()
            .unwrap()
            .current_tab_entity()
            .unwrap();
        let preview_frame_selected = world
            .get_resource::<AnimationManager>()
            .unwrap()
            .preview_frame_selected();

        match (
            preview_frame_selected,
            click_type,
            input_manager.selected_shape,
        ) {
            (false, MouseButton::Left, Some((vertex_2d_entity, CanvasShape::Vertex))) => {
                // move vertex
                let Some(vertex_3d_entity) = world.get_resource::<VertexManager>().unwrap().vertex_entity_2d_to_3d(&vertex_2d_entity) else {
                    warn!(
                        "Selected vertex entity: {:?} has no 3d counterpart",
                        vertex_2d_entity
                    );
                    return;
                };

                world.resource_scope(|world, mut animation_manager: Mut<AnimationManager>| {
                    animation_manager.drag_vertex(
                        world,
                        &current_file_entity,
                        vertex_3d_entity,
                        vertex_2d_entity,
                        mouse_position,
                    );
                });
            }
            (false, MouseButton::Left, Some((edge_2d_entity, CanvasShape::Edge))) => {
                // move edge
                let edge_3d_entity = world
                    .get_resource::<EdgeManager>()
                    .unwrap()
                    .edge_entity_2d_to_3d(&edge_2d_entity)
                    .unwrap();

                world.resource_scope(|world, mut animation_manager: Mut<AnimationManager>| {
                    animation_manager.drag_edge(
                        world,
                        &current_file_entity,
                        edge_3d_entity,
                        edge_2d_entity,
                        mouse_position,
                    );
                });
            }
            _ => InputManager::handle_drag_empty_space(world, click_type, delta),
        }
    }

    fn handle_mouse_scroll_framing(world: &mut World, scroll_y: f32) {
        let mut animation_manager = world.get_resource_mut::<AnimationManager>().unwrap();
        animation_manager.framing_handle_mouse_wheel(scroll_y);
    }

    pub(crate) fn sync_mouse_hover_ui(
        world: &mut World,
        current_file_entity: &Entity,
        mouse_position: &Vec2,
    ) -> Option<(Entity, CanvasShape)> {

        if world.get_resource::<AnimationManager>().unwrap().is_framing() {
            let canvas_size = world.get_resource::<Canvas>().unwrap().canvas_texture_size();
            world.get_resource_mut::<AnimationManager>().unwrap().sync_mouse_hover_ui_framing(
                current_file_entity,
                canvas_size,
                mouse_position,
            );
            return None;
        } else {
            Self::sync_mouse_hover_ui_posing(world, mouse_position)
        }
    }

    fn sync_mouse_hover_ui_posing(
        world: &mut World,
        mouse_position: &Vec2,
    ) -> Option<(Entity, CanvasShape)> {

        let mut system_state: SystemState<(
            Res<TabManager>,
            Res<VertexManager>,
            Res<EdgeManager>,
            Res<AnimationManager>,
            Query<(&Transform, Option<&LocalShape>)>,
            Query<&Visibility>,
            Query<&ShapeName>,
            Query<(Entity, Option<&VertexRoot>), (With<Vertex2d>, Without<LocalShape>)>,
            Query<(Entity, &Edge2dLocal), Without<LocalShape>>,
        )> = SystemState::new(world);
        let (
            tab_manager,
            vertex_manager,
            edge_manager,
            animation_manager,
            transform_q,
            visibility_q,
            shape_name_q,
            vertex_2d_q,
            edge_2d_q,
        ) = system_state.get_mut(world);

        let Some(current_tab_state) = tab_manager.current_tab_state() else {
            return None;
        };
        let camera_state = &current_tab_state.camera_state;
        if animation_manager.preview_frame_selected() {
            return None;
        }

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

            // don't hover over disabled vertices in Anim mode
            let vertex_3d_entity = vertex_manager
                .vertex_entity_2d_to_3d(&vertex_2d_entity)
                .unwrap();
            let Ok(shape_name) = shape_name_q.get(vertex_3d_entity) else { continue; };
            let shape_name = shape_name.value.as_str();
            if shape_name.len() == 0 {
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

                let edge_3d_entity = edge_manager.edge_entity_2d_to_3d(&edge_2d_entity).unwrap();
                let (_, end_vertex_3d_entity) = edge_manager.edge_get_endpoints(&edge_3d_entity);
                let Ok(shape_name) = shape_name_q.get(end_vertex_3d_entity) else { continue; };
                let shape_name = shape_name.value.as_str();
                if shape_name.len() == 0 {
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

        if is_hovering { least_entity } else { None }
    }

    fn reset_last_dragged_rotation(input_manager: &mut InputManager, world: &mut World) {
        // reset last dragged rotation
        if let Some((vertex_2d_entity, old_angle, new_angle)) = world
            .get_resource_mut::<AnimationManager>()
            .unwrap()
            .take_last_rotation_dragged()
        {
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                tab_manager.current_tab_execute_anim_action(
                    world,
                    input_manager,
                    AnimAction::RotateVertex(vertex_2d_entity, old_angle, Some(new_angle)),
                );
            });
        }
    }

    pub(crate) fn handle_insert_frame(world: &mut World, input_manager: &mut InputManager) {
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            let current_file_entity = *tab_manager.current_tab_entity().unwrap();
            let animation_manager = world.get_resource::<AnimationManager>().unwrap();
            let current_frame_index = animation_manager.current_frame_index();

            // copy all rotations from current frame
            let mut rotations = Vec::new();
            let current_frame_entity = animation_manager
                .current_frame_entity(&current_file_entity)
                .unwrap();
            let rotation_entities: Vec<Entity> = animation_manager
                .get_frame_rotations(&current_file_entity, &current_frame_entity)
                .unwrap()
                .iter()
                .copied()
                .collect();
            let mut rot_q = world.query::<&AnimRotation>();
            for rotation_entity in rotation_entities.iter() {
                let Ok(rot) = rot_q.get(world, *rotation_entity) else {
                    continue;
                };
                let name: String = (*rot.vertex_name).clone();
                let quat = rot.get_rotation();
                rotations.push((name, quat));
            }

            // execute insertion
            tab_manager.current_tab_execute_anim_action(
                world,
                input_manager,
                AnimAction::InsertFrame(
                    current_file_entity,
                    current_frame_index + 1,
                    Some(rotations),
                ),
            );
        });
    }

    pub(crate) fn handle_delete_frame(world: &mut World, input_manager: &mut InputManager) {
        let Some(current_file_entity) = world.get_resource::<TabManager>().unwrap().current_tab_entity() else {
            return;
        };
        let current_file_entity = *current_file_entity;

        let mut system_state: SystemState<(Commands, Client, Res<AnimationManager>)> =
            SystemState::new(world);
        let (mut commands, client, animation_manager) = system_state.get_mut(world);

        // delete vertex
        let Some(current_frame_entity) = animation_manager.current_frame_entity(&current_file_entity) else {
            return;
        };

        // check whether we can delete vertex
        let auth_status = commands
            .entity(current_frame_entity)
            .authority(&client)
            .unwrap();
        if !auth_status.is_granted() && !auth_status.is_available() {
            // do nothing, file is not available
            // TODO: queue for deletion? check before this?
            warn!(
                "Frame `{:?}` is not available for deletion!",
                current_frame_entity
            );
            return;
        }

        let current_frame_index = animation_manager.current_frame_index();

        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            tab_manager.current_tab_execute_anim_action(
                world,
                input_manager,
                AnimAction::DeleteFrame(current_file_entity, current_frame_index),
            );
        });
    }

    fn handle_play_pause(world: &mut World) {
        let mut animation_manager = world.get_resource_mut::<AnimationManager>().unwrap();
        if animation_manager.preview_is_playing() {
            animation_manager.preview_pause();
        } else {
            animation_manager.preview_play();
        }
    }
}
