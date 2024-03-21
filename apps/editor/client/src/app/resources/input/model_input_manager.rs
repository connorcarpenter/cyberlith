use std::f32::consts::FRAC_PI_2;

use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{Commands, Query, Res, ResMut, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt};

use input::{InputEvent, Key, MouseButton};
use math::{quat_from_spin_direction, spin_direction_from_quat, Vec2, Vec3};
use render_api::{
    components::{Camera, Projection, Transform},
    shapes::{angle_between, get_2d_line_transform_endpoint, normalize_angle},
};

use editor_proto::components::{
    EdgeAngle, FileExtension, NetTransform, ShapeName, Vertex3d, VertexRoot,
};

use crate::app::{
    components::{
        Edge2dLocal, EdgeAngleLocal, NetTransformControl, NetTransformControlType,
        NetTransformLocal, OwnedByFileLocal, ScaleAxis, Vertex2d,
    },
    get_new_3d_position,
    plugin::Main,
    resources::{
        action::model::ModelAction, camera_manager::CameraManager, canvas::Canvas,
        edge_manager::EdgeManager, file_manager::FileManager, input::InputManager,
        model_manager::ModelManager, shape_data::CanvasShape, tab_manager::TabManager,
    },
};

pub struct ModelInputManager;

impl ModelInputManager {
    pub fn update_input(
        world: &mut World,
        input_manager: &mut InputManager,
        file_ext: &FileExtension,
        input_actions: Vec<InputEvent>,
    ) {
        for action in input_actions {
            match action {
                InputEvent::MouseClicked(click_type, mouse_position) => {
                    Self::handle_mouse_click(world, input_manager, &mouse_position, click_type)
                }
                InputEvent::MouseDragged(click_type, mouse_position, delta) => {
                    Self::handle_mouse_drag(
                        world,
                        input_manager,
                        file_ext,
                        mouse_position,
                        delta,
                        click_type,
                    )
                }
                InputEvent::MouseMiddleScrolled(scroll_y) => {
                    InputManager::handle_mouse_scroll_wheel(world, scroll_y)
                }
                InputEvent::MouseMoved => {
                    input_manager.queue_resync_hover_ui();
                    input_manager.queue_resync_selection_ui();
                }
                InputEvent::MouseReleased(MouseButton::Left) => {
                    world.resource_scope(|world, mut model_manager: Mut<ModelManager>| {
                        model_manager.on_drag_transform_end(world, input_manager);
                    });
                }
                InputEvent::KeyPressed(key) => match key {
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
                Some(CanvasShape::RootVertex | CanvasShape::Face) | None,
            )
            | (MouseButton::Right, _, _) => {
                // deselect shape
                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    tab_manager.current_tab_execute_model_action(
                        world,
                        input_manager,
                        ModelAction::SelectShape(None),
                    );
                });
            }
            (MouseButton::Left, _, Some(CanvasShape::Vertex | CanvasShape::Edge)) => {
                // select hovered shape
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
        file_ext: &FileExtension,
        mouse_position: Vec2,
        delta: Vec2,
        click_type: MouseButton,
    ) {
        if !world.get_resource::<TabManager>().unwrap().has_focus() {
            return;
        }

        match (click_type, input_manager.selected_shape) {
            (MouseButton::Left, Some((transform_entity, shape))) => Self::handle_transform_drag(
                world,
                file_ext,
                &transform_entity,
                shape,
                &mouse_position,
            ),
            (_, _) => InputManager::handle_drag_empty_space(world, click_type, delta),
        }
    }

    pub(crate) fn sync_mouse_hover_ui(
        world: &mut World,
        file_ext: &FileExtension,
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
            Query<(Entity, &OwnedByFileLocal), With<Edge2dLocal>>,
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

        let skel_file_entity_opt = match file_ext {
            FileExtension::Model => {
                let Some(skel_file_entity) =
                    file_manager.file_get_dependency(current_file_entity, FileExtension::Skel)
                else {
                    return None;
                };
                Some(skel_file_entity)
            }
            FileExtension::Scene => None,
            _ => panic!("invalid"),
        };

        let mut least_distance = f32::MAX;
        let mut least_entity = None;
        let mut is_hovering = false;

        let ntc_2d_vertices = model_manager.net_transform_2d_vertices(current_file_entity);

        Self::handle_vertex_hover(
            &transform_q,
            ntc_2d_vertices,
            &vertex_root_q,
            camera_3d_scale,
            mouse_position,
            &mut least_distance,
            &mut least_entity,
            &mut is_hovering,
        );

        if let Some(skel_file_entity) = skel_file_entity_opt {
            // check skel edge entities
            let mut edge_2d_entities = Vec::new();
            for (edge_2d_entity, owned_by_local) in edge_2d_q.iter() {
                if owned_by_local.file_entity == skel_file_entity {
                    edge_2d_entities.push(edge_2d_entity);
                }
            }

            Self::handle_edge_hover(
                &transform_q,
                edge_2d_entities,
                Some((&edge_manager, &shape_name_q)),
                camera_3d_scale,
                mouse_position,
                &mut least_distance,
                &mut least_entity,
                &mut is_hovering,
            );
        }

        // check rotation edge entities
        let mut edge_2d_entities = Vec::new();
        let ntc_2d_edges =
            model_manager.net_transform_rotation_edge_2d_entities(current_file_entity);
        for ntc_2d_edge in ntc_2d_edges {
            edge_2d_entities.push(ntc_2d_edge);
        }

        Self::handle_edge_hover(
            &transform_q,
            edge_2d_entities,
            None,
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
            InputManager::hover_check_vertex(
                transform_q,
                None,
                mouse_position,
                least_distance,
                least_entity,
                &vertex_2d_entity,
                root_opt,
            );
        }

        *is_hovering = *least_distance <= (Vertex2d::DETECT_RADIUS * camera_3d_scale);
    }

    fn handle_edge_hover(
        transform_q: &Query<&Transform>,
        edge_2d_entities: Vec<Entity>,
        anim_opt: Option<(&EdgeManager, &Query<&ShapeName>)>,
        camera_3d_scale: f32,
        mouse_position: &Vec2,
        least_distance: &mut f32,
        least_entity: &mut Option<(Entity, CanvasShape)>,
        is_hovering: &mut bool,
    ) {
        // check for edges
        if !*is_hovering {
            for edge_2d_entity in edge_2d_entities {
                InputManager::hover_check_edge(
                    transform_q,
                    anim_opt,
                    mouse_position,
                    least_distance,
                    least_entity,
                    &edge_2d_entity,
                );
            }

            *is_hovering = *least_distance <= (Edge2dLocal::DETECT_THICKNESS * camera_3d_scale);
        }
    }

    fn handle_transform_drag(
        world: &mut World,
        file_ext: &FileExtension,
        control_2d_entity: &Entity,
        shape: CanvasShape,
        mouse_position: &Vec2,
    ) {
        let mut system_state: SystemState<(
            Commands,
            Client<Main>,
            Res<ModelManager>,
            Query<&Vertex3d>,
            Query<&EdgeAngle>,
            Query<&NetTransform>,
            Query<&NetTransformControl>,
        )> = SystemState::new(world);
        let (
            mut commands,
            client,
            model_manager,
            vertex_3d_q,
            edge_angle_q,
            net_transform_q,
            ntc_q,
        ) = system_state.get_mut(world);

        let Ok(ntc_component) = ntc_q.get(*control_2d_entity) else {
            warn!("Expected NTC");
            return;
        };
        let ntc_entity = ntc_component.net_transform_entity;
        let ntc_type = ntc_component.control_type;

        // get bone transform
        let bone_transform_opt = match file_ext {
            FileExtension::Model => {
                let Some(bone_transform) =
                    model_manager.get_bone_transform(&vertex_3d_q, &edge_angle_q, &ntc_entity)
                else {
                    return;
                };
                Some(bone_transform)
            }
            FileExtension::Scene => None,
            _ => panic!("invalid"),
        };

        // check status
        let auth_status = commands.entity(ntc_entity).authority(&client).unwrap();
        if !(auth_status.is_requested() || auth_status.is_granted()) {
            // only continue to mutate if requested or granted authority over ntc
            info!("No authority over ntc, skipping..");
            return;
        }

        // set networked 3d vertex position
        let net_transform = net_transform_q.get(ntc_entity).unwrap();

        let old_transform = NetTransformLocal::to_transform(&net_transform);
        let mut new_transform = old_transform;

        if let Some(bone_transform) = bone_transform_opt {
            // apply parent bone transform to new_transform
            new_transform = new_transform.multiply(&bone_transform);
        }

        match (shape, ntc_type) {
            (CanvasShape::Vertex, NetTransformControlType::Translation) => {
                let mut system_state: SystemState<(
                    Res<CameraManager>,
                    Query<(&Camera, &Projection)>,
                    Query<&Transform>,
                )> = SystemState::new(world);
                let (camera_manager, camera_q, transform_q) = system_state.get_mut(world);

                let new_3d_position = get_new_3d_position(
                    &camera_manager,
                    &camera_q,
                    &transform_q,
                    &mouse_position,
                    control_2d_entity,
                );
                new_transform.translation = new_3d_position;
            }
            (CanvasShape::Vertex, NetTransformControlType::RotationVertex) => {
                let rotation_edge_3d_entity = model_manager
                    .get_rotation_edge_3d_entity(&ntc_entity)
                    .unwrap();

                let mut system_state: SystemState<(
                    Res<CameraManager>,
                    Query<(&Camera, &Projection)>,
                    Query<&Transform>,
                    Query<&EdgeAngleLocal>,
                )> = SystemState::new(world);
                let (camera_manager, camera_q, transform_q, edge_angle_q) =
                    system_state.get_mut(world);

                let edge_angle = edge_angle_q.get(rotation_edge_3d_entity).unwrap();
                let edge_angle = edge_angle.get_radians();

                let new_3d_position = get_new_3d_position(
                    &camera_manager,
                    &camera_q,
                    &transform_q,
                    &mouse_position,
                    control_2d_entity,
                );
                let target_direction = (new_3d_position - new_transform.translation).normalize();
                let rotation_angle =
                    quat_from_spin_direction(edge_angle, Vec3::X, target_direction);
                new_transform.rotation = rotation_angle;
            }
            (CanvasShape::Edge, NetTransformControlType::RotationEdge) => {
                let mut system_state: SystemState<(
                    Res<EdgeManager>,
                    Query<&mut EdgeAngleLocal>,
                    Query<&Transform>,
                )> = SystemState::new(world);
                let (edge_manager, mut edge_angle_q, transform_q) = system_state.get_mut(world);

                let edge_3d_entity = edge_manager
                    .edge_entity_2d_to_3d(control_2d_entity)
                    .unwrap();
                let edge_2d_transform = transform_q.get(*control_2d_entity).unwrap();
                let start_pos = edge_2d_transform.translation.truncate();
                let end_pos = get_2d_line_transform_endpoint(&edge_2d_transform);
                let base_angle = angle_between(&start_pos, &end_pos);

                let edge_angle_entity = edge_manager.edge_get_base_circle_entity(&edge_3d_entity);
                let edge_angle_pos = transform_q
                    .get(edge_angle_entity)
                    .unwrap()
                    .translation
                    .truncate();

                let mut edge_angle = edge_angle_q.get_mut(edge_3d_entity).unwrap();
                let new_angle = normalize_angle(
                    angle_between(&edge_angle_pos, &mouse_position) - FRAC_PI_2 - base_angle,
                );

                edge_angle.set_radians(new_angle);

                // set transform
                let (_, old_direction) = spin_direction_from_quat(Vec3::X, new_transform.rotation);
                let rotation_angle = quat_from_spin_direction(new_angle, Vec3::X, old_direction);
                new_transform.rotation = rotation_angle;
            }
            (CanvasShape::Vertex, NetTransformControlType::Scale(axis)) => {
                let mut system_state: SystemState<(
                    Res<CameraManager>,
                    Query<(&Camera, &Projection)>,
                    Query<&Transform>,
                )> = SystemState::new(world);
                let (camera_manager, camera_q, transform_q) = system_state.get_mut(world);

                let translation = new_transform.translation;
                let old_scale = new_transform.scale;

                let new_3d_position = get_new_3d_position(
                    &camera_manager,
                    &camera_q,
                    &transform_q,
                    &mouse_position,
                    control_2d_entity,
                );

                let new_scale = match axis {
                    ScaleAxis::X => {
                        let mut output = old_scale;
                        let new_x = (new_3d_position.x - translation.x)
                            / NetTransformControl::SCALE_EDGE_LENGTH;
                        output.x = new_x;
                        output
                    }
                    ScaleAxis::Y => {
                        let mut output = old_scale;
                        let new_y = (new_3d_position.y - translation.y)
                            / NetTransformControl::SCALE_EDGE_LENGTH;
                        output.y = new_y;
                        output
                    }
                    ScaleAxis::Z => {
                        let mut output = old_scale;
                        let new_z = (new_3d_position.z - translation.z)
                            / NetTransformControl::SCALE_EDGE_LENGTH;
                        output.z = new_z;
                        output
                    }
                };
                new_transform.scale = new_scale;
            }
            _ => panic!("Unexpected NTC type"),
        }

        let mut system_state: SystemState<(
            ResMut<Canvas>,
            ResMut<ModelManager>,
            Query<&mut NetTransform>,
        )> = SystemState::new(world);
        let (mut canvas, mut model_manager, mut net_transform_q) = system_state.get_mut(world);

        if let Some(bone_transform) = bone_transform_opt {
            // remove parent transform from new_transform
            let mut bone_transform_inverse = bone_transform.inverse();
            bone_transform_inverse.scale = Vec3::ONE;
            new_transform = new_transform.multiply(&bone_transform_inverse);
        }

        let mut net_transform = net_transform_q.get_mut(ntc_entity).unwrap();
        NetTransformLocal::set_transform(&mut net_transform, &new_transform);

        model_manager.update_last_transform_dragged(ntc_entity, old_transform, new_transform);

        // redraw
        canvas.queue_resync_shapes();
    }
}
