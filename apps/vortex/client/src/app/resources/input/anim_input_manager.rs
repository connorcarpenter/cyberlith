use bevy_ecs::{
    entity::Entity,
    system::{Commands, Res, ResMut, SystemState},
    world::{Mut, World},
};
use bevy_log::warn;

use naia_bevy_client::{Client, CommandsExt, Instant};

use input::{InputAction, Key, MouseButton};
use math::Vec2;

use vortex_proto::components::AnimRotation;

use crate::app::resources::{
    action::animation::AnimAction,
    animation_manager::AnimationManager,
    canvas::Canvas,
    edge_manager::EdgeManager,
    input::{CardinalDirection, InputManager},
    shape_data::CanvasShape,
    tab_manager::TabManager,
    vertex_manager::VertexManager,
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
                    let mut animation_manager =
                        world.get_resource_mut::<AnimationManager>().unwrap();
                    animation_manager.framing_queue_resync_hover_ui();
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

    pub(crate) fn handle_mouse_click_framing(
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

    pub(crate) fn handle_mouse_click_posing(
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

        // click_type, selected_shape, hovered_shape, current_file_type
        match (click_type, selected_shape, hovered_shape) {
            (MouseButton::Left, Some(_), Some(shape)) => {
                match shape {
                    CanvasShape::Vertex | CanvasShape::Edge => {
                        // select hovered entity
                        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                            tab_manager.current_tab_execute_anim_action(
                                world,
                                input_manager,
                                AnimAction::SelectShape(input_manager.hovered_entity),
                            );
                        });
                        return;
                    }
                    _ => {
                        // deselect vertex
                        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                            tab_manager.current_tab_execute_anim_action(
                                world,
                                input_manager,
                                AnimAction::SelectShape(None),
                            );
                        });
                        return;
                    }
                }
            }
            (MouseButton::Left, Some(CanvasShape::Vertex), None) => {
                // deselect vertex
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_anim_action(
                        world,
                        input_manager,
                        AnimAction::SelectShape(None),
                    );
                });
                return;
            }
            (MouseButton::Left, None, Some(CanvasShape::Vertex | CanvasShape::Edge)) => {
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

    pub(crate) fn handle_mouse_drag_framing(
        world: &mut World,
        click_type: MouseButton,
        delta: Vec2,
    ) {
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

    pub(crate) fn handle_mouse_drag_posing(
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

        let shape_is_selected = input_manager.selected_shape.is_some();
        let shape_can_drag = shape_is_selected
            && match input_manager.selected_shape.unwrap().1 {
                CanvasShape::RootVertex | CanvasShape::Vertex | CanvasShape::Edge => true,
                _ => false,
            };
        let preview_frame_selected = world
            .get_resource::<AnimationManager>()
            .unwrap()
            .preview_frame_selected();

        if shape_is_selected && shape_can_drag && !preview_frame_selected {
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

                            world.resource_scope(
                                |world, mut animation_manager: Mut<AnimationManager>| {
                                    animation_manager.drag_vertex(
                                        world,
                                        &current_file_entity,
                                        vertex_3d_entity,
                                        vertex_2d_entity,
                                        mouse_position,
                                    );
                                },
                            );
                        }
                        (edge_2d_entity, CanvasShape::Edge) => {
                            let edge_3d_entity = world
                                .get_resource::<EdgeManager>()
                                .unwrap()
                                .edge_entity_2d_to_3d(&edge_2d_entity)
                                .unwrap();

                            world.resource_scope(
                                |world, mut animation_manager: Mut<AnimationManager>| {
                                    animation_manager.drag_edge(
                                        world,
                                        &current_file_entity,
                                        edge_3d_entity,
                                        edge_2d_entity,
                                        mouse_position,
                                    );
                                },
                            );
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

    fn handle_mouse_scroll_framing(world: &mut World, scroll_y: f32) {
        let mut animation_manager = world.get_resource_mut::<AnimationManager>().unwrap();
        animation_manager.framing_handle_mouse_wheel(scroll_y);
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
