use bevy_ecs::{
    entity::Entity,
    query::With,
    system::{Commands, Query, Res, ResMut, SystemState},
    world::{Mut, World},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use input::{InputAction, Key, MouseButton};
use math::{convert_2d_to_3d, quat_from_spin_direction, Vec2, Vec3};
use render_api::components::{Camera, CameraProjection, Projection, Transform};

use vortex_proto::components::{FileExtension, ModelTransform, ShapeName, VertexRoot};

use crate::app::{
    components::{ ScaleAxis, ModelTransformControlType,
        Edge2dLocal, ModelTransformControl, ModelTransformLocal, OwnedByFileLocal, Vertex2d,
    },
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
                    world.resource_scope(|world, mut model_manager: Mut<ModelManager>| {
                        model_manager.on_drag_transform_end(world, input_manager);
                    });
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
        mouse_position: Vec2,
        delta: Vec2,
        click_type: MouseButton,
    ) {
        if !world.get_resource::<TabManager>().unwrap().has_focus() {
            return;
        }

        match (click_type, input_manager.selected_shape) {
            (MouseButton::Left, Some((transform_entity, CanvasShape::Vertex))) => {
                Self::handle_transform_drag(world, &transform_entity, &mouse_position)
            }
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
                InputManager::hover_check_edge(
                    transform_q,
                    Some((edge_manager, shape_name_q)),
                    mouse_position,
                    least_distance,
                    least_entity,
                    &edge_2d_entity,
                );
            }

            *is_hovering = *least_distance <= (Edge2dLocal::DETECT_THICKNESS * camera_3d_scale);
        }
    }

    fn handle_transform_drag(world: &mut World, control_2d_entity: &Entity, mouse_position: &Vec2) {
        let mut system_state: SystemState<(
            Commands,
            Client,
            Res<CameraManager>,
            ResMut<ModelManager>,
            ResMut<Canvas>,
            Query<(&Camera, &Projection)>,
            Query<&Transform>,
            Query<&mut ModelTransform>,
            Query<&ModelTransformControl, With<Vertex2d>>,
        )> = SystemState::new(world);
        let (
            mut commands,
            client,
            camera_manager,
            mut model_manager,
            mut canvas,
            camera_q,
            transform_q,
            mut model_transform_q,
            mtc_q,
        ) = system_state.get_mut(world);

        let Ok(mtc_component) = mtc_q.get(*control_2d_entity) else {
            panic!("Expected MTC");
        };
        let mtc_entity = mtc_component.model_transform_entity;
        let mtc_type = mtc_component.control_type;

        // check status
        let auth_status = commands.entity(mtc_entity).authority(&client).unwrap();
        if !(auth_status.is_requested() || auth_status.is_granted()) {
            // only continue to mutate if requested or granted authority over mtc
            info!("No authority over mtc, skipping..");
            return;
        }

        // get camera
        let camera_3d = camera_manager.camera_3d_entity().unwrap();
        let camera_transform: Transform = *transform_q.get(camera_3d).unwrap();
        let (camera, camera_projection) = camera_q.get(camera_3d).unwrap();

        let camera_viewport = camera.viewport.unwrap();
        let view_matrix = camera_transform.view_matrix();
        let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

        // get 2d vertex transform
        let control_2d_transform = transform_q.get(*control_2d_entity).unwrap();

        // convert 2d to 3d
        let new_3d_position = convert_2d_to_3d(
            &view_matrix,
            &projection_matrix,
            &camera_viewport.size_vec2(),
            &mouse_position,
            control_2d_transform.translation.z,
        );

        // set networked 3d vertex position
        let mut model_transform = model_transform_q.get_mut(mtc_entity).unwrap();

        let old_transform = ModelTransformLocal::to_transform(&model_transform);

        match mtc_type {
            ModelTransformControlType::Translation => {
                model_transform.set_translation_vec3(&new_3d_position);
            }
            ModelTransformControlType::Rotation => {
                let edge_angle = 0.0; //todo, find edge angle

                let rotation_with_offset = new_3d_position;
                let translation = model_transform.translation_vec3();
                let rotation_vector = rotation_with_offset - translation;
                let base_direction = Vec3::Z;
                let target_direction = rotation_vector.normalize();
                let rotation_angle = quat_from_spin_direction(edge_angle, base_direction, target_direction);
                model_transform.set_rotation(rotation_angle);
            }
            ModelTransformControlType::Scale(axis) => {

                let translation = model_transform.translation_vec3();
                let old_scale = model_transform.scale_vec3();

                let new_scale = match axis {
                    ScaleAxis::X => {
                        let mut output = old_scale;
                        let new_x = (new_3d_position.x - translation.x) / ModelTransformControl::SCALE_EDGE_LENGTH;
                        output.x = new_x;
                        output
                    }
                    ScaleAxis::Y => {
                        let mut output = old_scale;
                        let new_y = (new_3d_position.y - translation.y) / ModelTransformControl::SCALE_EDGE_LENGTH;
                        output.y = new_y;
                        output
                    }
                    ScaleAxis::Z => {
                        let mut output = old_scale;
                        let new_z = (new_3d_position.z - translation.z) / ModelTransformControl::SCALE_EDGE_LENGTH;
                        output.z = new_z;
                        output
                    }
                };
                model_transform.set_scale_vec3(&new_scale);
            }
            _ => {
                panic!("Unexpected MTC type");
            }
        }

        let new_transform = ModelTransformLocal::to_transform(&model_transform);

        model_manager.update_last_transform_dragged(mtc_entity, old_transform, new_transform);

        // redraw
        canvas.queue_resync_shapes();
    }
}
