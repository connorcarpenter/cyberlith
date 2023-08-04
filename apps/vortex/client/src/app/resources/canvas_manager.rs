use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::Resource,
    query::{With, Without},
    system::{Commands, Query},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, Replicate};
use input::{Input, Key, MouseButton};
use math::{convert_2d_to_3d, convert_3d_to_2d, Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{
        Camera, CameraProjection, Projection, Transform,
        Visibility,
    },
    shapes::{distance_to_2d_line, get_2d_line_transform_endpoint, set_2d_line_transform},
    Assets,
};
use vortex_proto::{components::{OwnedByTab, Vertex3d, VertexRootChild}, types::TabId};

use crate::app::{
    components::{Compass, Edge2d, Edge3d, HoverCircle, SelectCircle, Vertex2d},
    resources::{action_stack::{Action, ActionStack}, camera_manager::CameraManager},
    set_3d_line_transform,
    systems::network::vertex_3d_postprocess,
};

#[derive(Clone, Copy)]
pub enum ClickType {
    Left,
    Right,
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CanvasShape {
    RootVertex,
    Vertex,
    Edge,
    // Face,
}

#[derive(Resource)]
pub struct CanvasManager {

    // input
    rotate_key_down: bool,
    click_type: ClickType,
    click_start: Vec2,
    click_down: bool,
    mouse_hover_recalc: bool,
    last_mouse_position: Vec2,

    // vertices
    vertices_3d_to_2d: HashMap<Entity, Entity>,
    vertices_2d_to_3d: HashMap<Entity, Entity>,

    vertices_recalc: bool,
    pub hover_circle_entity: Option<Entity>,
    hovered_entity: Option<(Entity, CanvasShape)>,
    last_vertex_dragged: Option<(Entity, Vec3, Vec3)>,

    pub select_circle_entity: Option<Entity>,
    pub select_line_entity: Option<Entity>,
    selected_vertex: Option<(Entity, CanvasShape)>,
    select_line_recalc: bool,
    compass_vertices: Vec<Entity>,
}

impl Default for CanvasManager {
    fn default() -> Self {
        Self {

            // input
            click_type: ClickType::Left,
            click_start: Vec2::ZERO,
            click_down: false,
            rotate_key_down: false,
            mouse_hover_recalc: false,
            last_mouse_position: Vec2::ZERO,

            // vertices
            vertices_3d_to_2d: HashMap::new(),
            vertices_2d_to_3d: HashMap::new(),

            vertices_recalc: false,

            hover_circle_entity: None,
            hovered_entity: None,
            last_vertex_dragged: None,
            select_circle_entity: None,
            select_line_entity: None,
            selected_vertex: None,
            select_line_recalc: false,
            compass_vertices: Vec::new(),
        }
    }
}

impl CanvasManager {
    pub fn update_input(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        camera_manager: &mut CameraManager,
        input: &mut Input,
        action_stack: &mut ActionStack,
        current_tab_id: TabId,
        transform_q: &mut Query<&mut Transform>,
        camera_q: &mut Query<(&mut Camera, &mut Projection)>,
        visibility_q: &mut Query<&mut Visibility>,
        owned_by_q: &Query<&OwnedByTab>,
        vertex_3d_q: &mut Query<&mut Vertex3d>,
        vertex_2d_q: &Query<(Entity, Option<&VertexRootChild>), (With<Vertex2d>, Without<Compass>)>,
        edge_2d_q: &Query<(Entity, &Edge2d), Without<Compass>>,
    ) {
        // Mouse wheel zoom..
        let scroll_y = input.consume_mouse_scroll();
        if scroll_y > 0.1 || scroll_y < -0.1 {
            camera_manager.camera_zoom(scroll_y);
        }

        // Mouse over
        if !self.click_down {
            self.update_mouse_hover(
                camera_manager,
                current_tab_id,
                input.mouse_position(),
                transform_q,
                visibility_q,
                owned_by_q,
                vertex_2d_q,
                edge_2d_q,
            );
        }
        self.update_select_line(input.mouse_position(), camera_manager, transform_q, visibility_q);

        // check keyboard input

        // (S)olid 3D View
        if input.is_pressed(Key::S) {
            // disable 2d camera, enable 3d camera
            camera_manager.set_3d_mode(camera_q);
            self.recalculate_vertices();
        }
        // (W)ireframe 2D View
        else if input.is_pressed(Key::W) {
            // disable 3d camera, enable 2d camera
            camera_manager.set_2d_mode(camera_q);
            self.recalculate_vertices();
        }
        // 1 Game Camera View
        else if input.is_pressed(Key::Num1) {
            camera_manager.set_camera_angle_ingame(1);
        }
        // 2 Game Camera View
        else if input.is_pressed(Key::Num2) {
            camera_manager.set_camera_angle_ingame(2);
        }
        // 3 Game Camera View
        else if input.is_pressed(Key::Num3) {
            camera_manager.set_camera_angle_ingame(3);
        }
        // 4 Game Camera View
        else if input.is_pressed(Key::Num4) {
            camera_manager.set_camera_angle_ingame(4);
        }
        // 5 Game Camera View
        else if input.is_pressed(Key::Num5) {
            camera_manager.set_camera_angle_ingame(5);
        }
        // Si(d)e Camera View
        else if input.is_pressed(Key::D) {
            camera_manager.set_camera_angle_side();
        }
        // (F)ront Camera View
        else if input.is_pressed(Key::F) {
            camera_manager.set_camera_angle_front();
        }
        // (T)op Camera View
        else if input.is_pressed(Key::T) {
            camera_manager.set_camera_angle_top();
        }
        // Delete
        else if input.is_pressed(Key::Delete) {
            self.handle_delete_key_press(commands, client, action_stack);
        }

        if !self.rotate_key_down {
            // Rotate Yaw 45 degrees
            if input.is_pressed(Key::PageUp) {
                camera_manager.set_camera_angle_yaw_rotate(true);
                self.rotate_key_down = true;
            }
            // Rotate Yaw 45 degrees
            else if input.is_pressed(Key::PageDown) {
                camera_manager.set_camera_angle_yaw_rotate(false);
                self.rotate_key_down = true;
            }
        } else {
            if !input.is_pressed(Key::PageUp) && !input.is_pressed(Key::PageDown) {
                self.rotate_key_down = false;
            }
        }

        // mouse clicks

        let left_button_pressed = input.is_pressed(MouseButton::Left);
        let right_button_pressed = input.is_pressed(MouseButton::Right);
        let mouse_button_pressed = left_button_pressed || right_button_pressed;

        if mouse_button_pressed {
            if left_button_pressed {
                self.click_type = ClickType::Left;
            }
            if right_button_pressed {
                self.click_type = ClickType::Right;
            }

            if self.click_down {
                // already clicking
                let mouse = *input.mouse_position();
                let delta = mouse - self.click_start;
                self.click_start = mouse;

                if delta.length() > 0.0 {
                    self.handle_mouse_drag(
                        commands,
                        client,
                        camera_manager,
                        self.click_type,
                        mouse,
                        delta,
                        camera_q,
                        transform_q,
                        vertex_3d_q,
                    );
                }
            } else {
                // haven't clicked yet
                let mouse = *input.mouse_position();
                self.click_down = true;
                self.click_start = mouse;
                self.handle_mouse_click(
                    camera_manager,
                    action_stack,
                    self.click_type,
                    &mouse,
                    camera_q,
                    transform_q,
                );
            }
        } else {
            if self.click_down {
                // release click
                self.click_down = false;

                if let Some((vertex_2d_entity, old_pos, new_pos)) = self.last_vertex_dragged.take()
                {
                    action_stack.buffer_action(Action::MoveVertex(
                        vertex_2d_entity,
                        old_pos,
                        new_pos,
                    ));
                }
            }
        }
    }

    pub fn sync_vertices(
        &mut self,
        camera_manager: &CameraManager,
        current_tab_id: TabId,
        transform_q: &mut Query<(&mut Transform, Option<&Compass>)>,
        camera_q: &Query<(&Camera, &Projection)>,
        vertex_3d_q: &mut Query<(Entity, &mut Vertex3d)>,
        edge_2d_q: &Query<(Entity, &Edge2d)>,
        edge_3d_q: &Query<(Entity, &Edge3d)>,
        owned_by_q: &Query<&OwnedByTab>,
    ) {
        if !self.vertices_recalc {
            return;
        }
        self.vertices_recalc = false;

        self.mouse_hover_recalc = true;
        self.select_line_recalc = true;

        let Some(camera_3d) = camera_manager.camera_3d_entity() else {
            return;
        };

        let Ok((camera_transform, _)) = transform_q.get(camera_3d) else {
            return;
        };

        self.compass_recalc(camera_manager, vertex_3d_q, &camera_transform);

        let Ok((camera, camera_projection)) = camera_q.get(camera_3d) else {
            return;
        };

        let camera_viewport = camera.viewport.unwrap();
        let view_matrix = camera_transform.view_matrix();
        let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

        // update vertices
        for (vertex_3d_entity, vertex_3d) in vertex_3d_q.iter() {

            // check if vertex is owned by the current tab
            if !Self::is_owned_by_tab_or_unowned(current_tab_id, owned_by_q, vertex_3d_entity) {
                continue;
            }

            // get transform
            let Ok((mut vertex_3d_transform, compass_opt)) = transform_q.get_mut(vertex_3d_entity) else {
                warn!("Vertex3d entity {:?} has no Transform", vertex_3d_entity);
                continue;
            };

            // update 3d vertices
            vertex_3d_transform.translation.x = vertex_3d.x().into();
            vertex_3d_transform.translation.y = vertex_3d.y().into();
            vertex_3d_transform.translation.z = vertex_3d.z().into();

            if compass_opt.is_some() {
                let scale_3d = Vertex2d::RADIUS / camera_manager.camera_3d_scale();
                vertex_3d_transform.scale = Vec3::splat(scale_3d);
            }

            let (coords, depth) = convert_3d_to_2d(
                &view_matrix,
                &projection_matrix,
                &camera_viewport.size_vec2(),
                &vertex_3d_transform.translation,
            );

            let Some(vertex_2d_entity) = self.vertex_entity_3d_to_2d(&vertex_3d_entity) else {
                panic!("Vertex3d entity {:?} has no corresponding Vertex2d entity", vertex_3d_entity);
            };
            let Ok((mut vertex_2d_transform, compass_opt)) = transform_q.get_mut(*vertex_2d_entity) else {
                panic!("Vertex2d entity {:?} has no Transform", vertex_2d_entity);
            };

            // update 2d vertices
            vertex_2d_transform.translation.x = coords.x;
            vertex_2d_transform.translation.y = coords.y;
            vertex_2d_transform.translation.z = depth;

            // update 2d compass
            if compass_opt.is_none() {
                let scale_2d = camera_manager.camera_3d_scale() * Vertex2d::RADIUS;
                vertex_2d_transform.scale = Vec3::splat(scale_2d);
            }

            // update hover circle
            if let Some((hover_entity, _)) = self.hovered_entity {
                if hover_entity == *vertex_2d_entity {
                    let hover_circle_entity = self.hover_circle_entity.unwrap();
                    let (mut hover_circle_transform, _) =
                        transform_q.get_mut(hover_circle_entity).unwrap();
                    hover_circle_transform.translation.x = coords.x;
                    hover_circle_transform.translation.y = coords.y;
                }
            }
        }

        // update 2d edges
        for (edge_entity, edge_endpoints) in edge_2d_q.iter() {
            let Some(end_entity) = self.vertex_entity_3d_to_2d(&edge_endpoints.end_3d) else {
                warn!("Edge entity {:?} has no endpoint entity", edge_entity);
                continue;
            };

            // check if vertex is owned by the current tab
            if !Self::is_owned_by_tab_or_unowned(current_tab_id, owned_by_q, *end_entity) {
                continue;
            }

            let (start_transform, _) = transform_q
                .get(edge_endpoints.start)
                .unwrap();
            let start_pos = start_transform
                .translation
                .truncate();

            let (end_transform, _) = transform_q.get(*end_entity).unwrap();
            let end_pos = end_transform.translation.truncate();

            let (mut edge_transform, _) = transform_q.get_mut(edge_entity).unwrap();
            set_2d_line_transform(&mut edge_transform, start_pos, end_pos);

            if let Some((hover_entity, CanvasShape::Edge)) = self.hovered_entity {
                if hover_entity == edge_entity {
                    edge_transform.scale.y = 3.0;
                }
            }
        }

        // update 3d edges
        for (edge_entity, edge_endpoints) in edge_3d_q.iter() {

            // check if vertex is owned by the current tab
            if !Self::is_owned_by_tab_or_unowned(current_tab_id, owned_by_q, edge_entity) {
                continue;
            }

            if let Ok((start_transform, _)) = transform_q.get(edge_endpoints.start) {
                let start_pos = start_transform.translation;
                if let Ok((end_transform, _)) = transform_q.get(edge_endpoints.end) {
                    let end_pos = end_transform.translation;
                    let (mut edge_transform, compass_opt) = transform_q.get_mut(edge_entity).unwrap();
                    set_3d_line_transform(&mut edge_transform, start_pos, end_pos);
                    if compass_opt.is_some() {
                        let scale_3d = 1.0 / camera_manager.camera_3d_scale();
                        edge_transform.scale.x = scale_3d;
                        edge_transform.scale.y = scale_3d;
                    }
                } else {
                    warn!(
                        "3d Edge end entity {:?} has no transform",
                        edge_endpoints.end
                    );
                }
            } else {
                warn!(
                    "3d Edge start entity {:?} has no transform",
                    edge_endpoints.start
                );
            }
        }
    }

    pub fn recalculate_vertices(&mut self) {
        self.vertices_recalc = true;
    }

    pub fn select_vertex(&mut self, entity: &Entity, shape: CanvasShape) {
        self.selected_vertex = Some((*entity, shape));

        self.select_line_recalc = true;
    }

    pub fn deselect_vertex(&mut self) {
        self.selected_vertex = None;

        self.select_line_recalc = true;
    }

    pub fn selected_vertex_2d(&self) -> Option<(Entity, CanvasShape)> {
        self.selected_vertex
    }

    fn handle_delete_key_press(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        action_stack: &mut ActionStack,
    ) {
        if self.selected_vertex.is_none() {
            return;
        }

        // delete vertex
        let (vertex_2d_entity, shape) = self.selected_vertex.unwrap();

        if shape == CanvasShape::RootVertex {
            return;
        }

        let vertex_3d_entity = self.vertex_entity_2d_to_3d(&vertex_2d_entity).unwrap();

        // check whether we can delete vertex
        let auth_status = commands
            .entity(*vertex_3d_entity)
            .authority(client)
            .unwrap();
        if !auth_status.is_granted() && !auth_status.is_available() {
            // do nothing, vertex is not available
            // TODO: queue for deletion? check before this?
            warn!(
                "Vertex {:?} is not available for deletion!",
                vertex_3d_entity
            );
            return;
        }

        let auth_status = commands
            .entity(*vertex_3d_entity)
            .authority(client)
            .unwrap();
        if !auth_status.is_granted() {
            // request authority if needed
            commands.entity(*vertex_3d_entity).request_authority(client);
        }

        action_stack.buffer_action(Action::DeleteVertex(vertex_2d_entity, None));

        self.selected_vertex = None;
    }

    fn update_mouse_hover(
        &mut self,
        camera_manager: &CameraManager,
        current_tab_id: TabId,
        mouse_position: &Vec2,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &mut Query<&mut Visibility>,
        owned_by_q: &Query<&OwnedByTab>,
        vertex_2d_q: &Query<(Entity, Option<&VertexRootChild>), (With<Vertex2d>, Without<Compass>)>,
        edge_2d_q: &Query<(Entity, &Edge2d), Without<Compass>>,
    ) {
        if mouse_position.x as i16 != self.last_mouse_position.x as i16
            || mouse_position.y as i16 != self.last_mouse_position.y as i16
        {
            // mouse moved!
            self.mouse_hover_recalc = true;
            self.select_line_recalc = true;
            self.last_mouse_position = *mouse_position;
        }

        if !self.mouse_hover_recalc {
            return;
        }

        self.mouse_hover_recalc = false;

        let mut least_distance = f32::MAX;
        let mut least_coords = Vec2::ZERO;
        let mut least_entity = None;

        for (vertex_entity, root_opt) in vertex_2d_q.iter() {

            // check tab ownership, skip vertices from other tabs
            if !Self::is_owned_by_tab(current_tab_id, owned_by_q, vertex_entity) {
                continue;
            }

            let vertex_transform = transform_q.get(vertex_entity).unwrap();
            let vertex_position = vertex_transform.translation.truncate();
            let distance = vertex_position.distance(*mouse_position);
            if distance < least_distance {
                least_distance = distance;
                least_coords = vertex_position;

                let shape = match root_opt {
                    Some(_) => CanvasShape::RootVertex,
                    None => CanvasShape::Vertex,
                };

                least_entity = Some((vertex_entity, shape));
            }
        }

        let mut is_hovering = least_distance <= (HoverCircle::DETECT_RADIUS * camera_manager.camera_3d_scale());

        // just setting edge thickness back to normal ... better way to do this?
        for (edge_entity, _) in edge_2d_q.iter() {
            let mut edge_transform = transform_q.get_mut(edge_entity).unwrap();
            edge_transform.scale.y = camera_manager.camera_3d_scale();
        }

        if !is_hovering {
            for (edge_entity, _) in edge_2d_q.iter() {

                // check tab ownership, skip edges from other tabs
                if !Self::is_owned_by_tab(current_tab_id, owned_by_q, edge_entity) {
                    continue;
                }

                let edge_transform = transform_q.get(edge_entity).unwrap();
                let edge_start = edge_transform.translation.truncate();
                let edge_end = get_2d_line_transform_endpoint(&edge_transform);

                let distance = distance_to_2d_line(*mouse_position, edge_start, edge_end);
                if distance < least_distance {
                    least_distance = distance;
                    least_entity = Some((edge_entity, CanvasShape::Edge));
                }
            }

            is_hovering = least_distance <= (Edge2d::HOVER_THICKNESS * camera_manager.camera_3d_scale());
        }

        let hover_circle_entity = self.hover_circle_entity.unwrap();
        let Ok(mut hover_circle_visibility) = visibility_q.get_mut(hover_circle_entity) else {
            panic!("HoverCircle entity has no Transform or Visibility");
        };

        if is_hovering {
            self.hovered_entity = least_entity;

            match self.hovered_entity {
                Some((_, CanvasShape::Vertex)) | Some((_, CanvasShape::RootVertex)) => {
                    // hovering over vertex
                    let Ok(mut hover_circle_transform) = transform_q.get_mut(hover_circle_entity) else {
                        panic!("HoverCircle entity has no Transform");
                    };
                    hover_circle_transform.translation.x = least_coords.x;
                    hover_circle_transform.translation.y = least_coords.y;
                    hover_circle_transform.scale =
                        Vec3::splat(HoverCircle::DISPLAY_RADIUS * camera_manager.camera_3d_scale());

                    hover_circle_visibility.visible = true;
                }
                Some((entity, CanvasShape::Edge)) => {
                    // hovering over edge
                    let Ok(mut edge_transform) = transform_q.get_mut(entity) else {
                        panic!("Edge entity has no Transform");
                    };
                    edge_transform.scale.y = Edge2d::HOVER_THICKNESS * camera_manager.camera_3d_scale();

                    hover_circle_visibility.visible = false;
                }
                None => {
                    todo!();
                }
            }
        } else {
            self.hovered_entity = None;
            hover_circle_visibility.visible = false;
        }
    }

    fn update_select_line(
        &mut self,
        mouse_position: &Vec2,
        camera_manager: &CameraManager,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &mut Query<&mut Visibility>,
    ) {
        if !self.select_line_recalc {
            return;
        }
        self.select_line_recalc = false;

        // update selected vertex line

        let select_line_entity = self.select_line_entity.unwrap();
        let select_circle_entity = self.select_circle_entity.unwrap();

        //

        // update selected vertex circle & line
        let Ok(mut select_shape_visibilities) = visibility_q.get_many_mut([select_circle_entity, select_line_entity]) else {
            panic!("Select shape entities has no Visibility");
        };

        if let Some((selected_vertex_entity, _)) = self.selected_vertex {
            let vertex_transform = {
                let Ok(vertex_transform) = transform_q.get(selected_vertex_entity) else {
                    return;
                };
                *vertex_transform
            };

            // sync select line transform
            {
                let Ok(mut select_line_transform) = transform_q.get_mut(select_line_entity) else {
                    panic!("Select line entity has no Transform");
                };

                set_2d_line_transform(
                    &mut select_line_transform,
                    vertex_transform.translation.truncate(),
                    *mouse_position,
                );
                select_line_transform.scale.y = camera_manager.camera_3d_scale();
            }

            // sync select circle transform
            {
                let Ok(mut select_circle_transform) = transform_q.get_mut(select_circle_entity) else {
                    panic!("Select shape entities has no Transform");
                };

                select_circle_transform.translation = vertex_transform.translation;
                select_circle_transform.scale =
                    Vec3::splat(SelectCircle::RADIUS * camera_manager.camera_3d_scale());
            }

            select_shape_visibilities[0].visible = true;
            select_shape_visibilities[1].visible = true;
        } else {
            select_shape_visibilities[0].visible = false;
            select_shape_visibilities[1].visible = false;
        }

        //

        if let Some((selected_vertex_entity, _)) = self.selected_vertex {
            let vertex_transform = {
                let Ok(vertex_transform) = transform_q.get(selected_vertex_entity) else {
                    return;
                };
                *vertex_transform
            };

            let Ok(mut select_line_transform) = transform_q.get_mut(select_line_entity) else {
                panic!("Select line entity has no Transform");
            };

            set_2d_line_transform(
                &mut select_line_transform,
                vertex_transform.translation.truncate(),
                *mouse_position,
            );
            select_line_transform.scale.y = camera_manager.camera_3d_scale();
        } else {
            let mut select_line_visibility = visibility_q.get_mut(select_line_entity).unwrap();
            select_line_visibility.visible = false;
        }
    }

    fn handle_mouse_click(
        &mut self,
        camera_manager: &CameraManager,
        action_stack: &mut ActionStack,
        click_type: ClickType,
        mouse_position: &Vec2,
        camera_q: &Query<(&mut Camera, &mut Projection)>,
        transform_q: &Query<&mut Transform>,
    ) {
        let cursor_is_hovering = self.hovered_entity.is_some();
        let vertex_is_selected = self.selected_vertex.is_some();

        if vertex_is_selected {
            match click_type {
                ClickType::Left => {
                    if cursor_is_hovering {
                        return;
                    }

                    // create new vertex

                    // get camera
                    let camera_3d = camera_manager.camera_3d_entity().unwrap();
                    let camera_transform: Transform = *transform_q.get(camera_3d).unwrap();
                    let (camera, camera_projection) = camera_q.get(camera_3d).unwrap();

                    let camera_viewport = camera.viewport.unwrap();
                    let view_matrix = camera_transform.view_matrix();
                    let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

                    // get 2d vertex transform
                    let (vertex_2d_entity, _) = self.selected_vertex.unwrap();
                    if let Ok(vertex_2d_transform) = transform_q.get(vertex_2d_entity) {
                        // convert 2d to 3d
                        let new_3d_position = convert_2d_to_3d(
                            &view_matrix,
                            &projection_matrix,
                            &camera_viewport.size_vec2(),
                            &mouse_position,
                            vertex_2d_transform.translation.z,
                        );

                        // spawn new vertex
                        action_stack.buffer_action(Action::CreateVertex(
                            vertex_2d_entity,
                            new_3d_position,
                            None,
                            None,
                        ));
                    } else {
                        warn!(
                            "Selected vertex entity: {:?} has no Transform",
                            vertex_2d_entity
                        );
                    }
                }
                ClickType::Right => {
                    if self.selected_vertex.is_none() {
                        return;
                    }

                    // deselect vertex
                    action_stack.buffer_action(Action::SelectVertex(None));
                }
            }
        } else {
            if cursor_is_hovering {
                match (self.hovered_entity.map(|(_, s)| s).unwrap(), click_type) {
                    (CanvasShape::Vertex, ClickType::Left)
                    | (CanvasShape::RootVertex, ClickType::Left) => {
                        action_stack.buffer_action(Action::SelectVertex(self.hovered_entity));
                    }
                    (CanvasShape::Vertex, ClickType::Right)
                    | (CanvasShape::RootVertex, ClickType::Right) => {
                        // do nothing, vertex deselection happens above
                    }
                    (CanvasShape::Edge, ClickType::Left) => { /* ? */ }
                    (CanvasShape::Edge, ClickType::Right) => {
                        // TODO: delete edge?
                    }
                }
            }
        }
    }

    fn handle_mouse_drag(
        &mut self,
        commands: &mut Commands,
        client: &Client,
        camera_manager: &mut CameraManager,
        click_type: ClickType,
        mouse_position: Vec2,
        delta: Vec2,
        camera_q: &Query<(&mut Camera, &mut Projection)>,
        transform_q: &Query<&mut Transform>,
        vertex_3d_q: &mut Query<&mut Vertex3d>,
    ) {
        let vertex_is_selected = self.selected_vertex.is_some();
        let vertex_is_root_vertex =
            vertex_is_selected && self.selected_vertex.unwrap().1 == CanvasShape::RootVertex;

        if vertex_is_selected && !vertex_is_root_vertex {
            match click_type {
                ClickType::Left => {
                    // move vertex
                    let (vertex_2d_entity, _) = self.selected_vertex.unwrap();

                    if let Some(vertex_3d_entity) = self.vertex_entity_2d_to_3d(&vertex_2d_entity) {
                        let auth_status = commands
                            .entity(*vertex_3d_entity)
                            .authority(client)
                            .unwrap();
                        if !(auth_status.is_requested() || auth_status.is_granted()) {
                            // only continue to mutate if requested or granted authority over vertex
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
                        let mut vertex_3d = vertex_3d_q.get_mut(*vertex_3d_entity).unwrap();

                        if let Some((_, old_3d_position, _)) = self.last_vertex_dragged {
                            self.last_vertex_dragged =
                                Some((vertex_2d_entity, old_3d_position, new_3d_position));
                        } else {
                            let old_3d_position = vertex_3d.as_vec3();
                            self.last_vertex_dragged =
                                Some((vertex_2d_entity, old_3d_position, new_3d_position));
                        }

                        vertex_3d.set_x(new_3d_position.x as i16);
                        vertex_3d.set_y(new_3d_position.y as i16);
                        vertex_3d.set_z(new_3d_position.z as i16);

                        // redraw
                        self.recalculate_vertices();
                    } else {
                        warn!(
                            "Selected vertex entity: {:?} has no 3d counterpart",
                            vertex_2d_entity
                        );
                    }
                }
                ClickType::Right => {
                    // TODO: dunno if this is possible? shouldn't the vertex be deselected?
                }
            }
        } else {
            match click_type {
                ClickType::Left => {
                    camera_manager.camera_pan(delta);
                }
                ClickType::Right => {
                    camera_manager.camera_orbit(delta);
                }
            }
        }
    }

    pub fn register_3d_vertex(&mut self, entity_3d: Entity, entity_2d: Entity) {
        self.vertices_3d_to_2d.insert(entity_3d, entity_2d);
        self.vertices_2d_to_3d.insert(entity_2d, entity_3d);
    }

    fn unregister_3d_vertex(&mut self, entity_3d: &Entity) -> Option<Entity> {
        if let Some(entity_2d) = self.vertices_3d_to_2d.remove(entity_3d) {
            self.vertices_2d_to_3d.remove(&entity_2d);
            return Some(entity_2d);
        }
        return None;
    }

    pub fn cleanup_deleted_vertex(
        &mut self,
        entity_3d: &Entity,
        commands: &mut Commands,
        edge_2d_q: &Query<(Entity, &Edge2d)>,
        edge_3d_q: &Query<(Entity, &Edge3d)>,
    ) {
        // despawn 3d edge
        for (edge_3d_entity, edge_3d) in edge_3d_q.iter() {
            if edge_3d.start == *entity_3d {
                info!("despawn 3d edge {:?}", edge_3d_entity);
                commands.entity(edge_3d_entity).despawn();
            }
        }

        if let Some(vertex_2d_entity) = self.unregister_3d_vertex(entity_3d) {
            // despawn 2d vertex
            info!("despawn 2d vertex {:?}", vertex_2d_entity);
            commands.entity(vertex_2d_entity).despawn();

            // despawn 2d edge
            for (edge_2d_entity, edge_2d) in edge_2d_q.iter() {
                if edge_2d.start == vertex_2d_entity {
                    info!("despawn 2d edge {:?}", edge_2d_entity);
                    commands.entity(edge_2d_entity).despawn();
                }
            }
        } else {
            panic!(
                "Vertex3d entity: `{:?}` has no corresponding Vertex2d entity",
                entity_3d
            );
        }

        self.recalculate_vertices();
    }

    pub(crate) fn vertex_entity_3d_to_2d(&self, entity_3d: &Entity) -> Option<&Entity> {
        self.vertices_3d_to_2d.get(entity_3d)
    }

    pub(crate) fn vertex_entity_2d_to_3d(&self, entity_2d: &Entity) -> Option<&Entity> {
        self.vertices_2d_to_3d.get(entity_2d)
    }

    pub(crate) fn setup_compass(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
    ) {
        let (root_vertex_2d_entity, vertex_3d_entity, _, _) =
            self.new_local_vertex(commands, camera_manager, meshes, materials, None, Vec3::ZERO, Color::WHITE);
        self.compass_vertices.push(vertex_3d_entity);
        commands.entity(root_vertex_2d_entity).insert(Compass);
        commands.entity(vertex_3d_entity).insert(Compass);

        let (vertex_2d_entity, vertex_3d_entity, Some(edge_2d_entity), Some(edge_3d_entity)) = self.new_local_vertex(
            commands,
            camera_manager,
            meshes,
            materials,
            Some(root_vertex_2d_entity),
            Vec3::new(100.0, 0.0, 0.0),
            Color::RED,
        ) else {
            panic!("No edges?");
        };
        self.compass_vertices.push(vertex_3d_entity);
        commands.entity(vertex_2d_entity).insert(Compass);
        commands.entity(vertex_3d_entity).insert(Compass);
        commands.entity(edge_2d_entity).insert(Compass);
        commands.entity(edge_3d_entity).insert(Compass);

        let (vertex_2d_entity, vertex_3d_entity, Some(edge_2d_entity), Some(edge_3d_entity)) = self.new_local_vertex(
            commands,
            camera_manager,
            meshes,
            materials,
            Some(root_vertex_2d_entity),
            Vec3::new(0.0, 100.0, 0.0),
            Color::GREEN,
        ) else {
            panic!("No edges?");
        };
        self.compass_vertices.push(vertex_3d_entity);
        commands.entity(vertex_2d_entity).insert(Compass);
        commands.entity(vertex_3d_entity).insert(Compass);
        commands.entity(edge_2d_entity).insert(Compass);
        commands.entity(edge_3d_entity).insert(Compass);

        let (vertex_2d_entity, vertex_3d_entity, Some(edge_2d_entity), Some(edge_3d_entity)) = self.new_local_vertex(
            commands,
            camera_manager,
            meshes,
            materials,
            Some(root_vertex_2d_entity),
            Vec3::new(0.0, 0.0, 100.0),
            Color::LIGHT_BLUE,
        ) else {
            panic!("No edges?");
        };
        self.compass_vertices.push(vertex_3d_entity);
        commands.entity(vertex_2d_entity).insert(Compass);
        commands.entity(vertex_3d_entity).insert(Compass);
        commands.entity(edge_2d_entity).insert(Compass);
        commands.entity(edge_3d_entity).insert(Compass);
    }

    pub(crate) fn setup_grid(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
    ) {
        self.new_grid_corner(commands, camera_manager, meshes, materials, true, true, true);
        self.new_grid_corner(commands, camera_manager, meshes, materials, true, false, false);

        self.new_grid_corner(commands, camera_manager, meshes, materials, false, true, false);
        self.new_grid_corner(commands, camera_manager, meshes, materials, false, false, true);
    }

    fn new_grid_corner(&mut self, commands: &mut Commands,
                       camera_manager: &mut CameraManager,
                       meshes: &mut Assets<CpuMesh>,
                       materials: &mut Assets<CpuMaterial>, x: bool, y: bool, z: bool) {

        let xf = if x { 1.0 } else { -1.0 };
        let yf = if y { 1.0 } else { -1.0 };
        let zf = if z { 1.0 } else { -1.0 };

        let grid_size: f32 = 100.0;
        let neg_grid_size: f32 = -grid_size;

        let (root_vertex_2d_entity, _, _, _) =
            self.new_local_vertex(commands, camera_manager, meshes, materials, None, Vec3::new(grid_size * xf, (grid_size * yf) + grid_size, grid_size * zf), Color::DARK_GRAY);
        commands.entity(root_vertex_2d_entity).insert(Compass);

        self.new_grid_vertex(commands, camera_manager, meshes, materials, root_vertex_2d_entity, Vec3::new(neg_grid_size * xf, (grid_size * yf) + grid_size, grid_size * zf));
        self.new_grid_vertex(commands, camera_manager, meshes, materials, root_vertex_2d_entity, Vec3::new(grid_size * xf, (neg_grid_size * yf) + grid_size, grid_size * zf));
        self.new_grid_vertex(commands, camera_manager, meshes, materials, root_vertex_2d_entity, Vec3::new(grid_size * xf, (grid_size * yf) + grid_size, neg_grid_size * zf));
    }

    fn new_grid_vertex(&mut self, commands: &mut Commands,
                       camera_manager: &mut CameraManager,
                       meshes: &mut Assets<CpuMesh>,
                       materials: &mut Assets<CpuMaterial>,
                       parent_vertex_2d_entity: Entity,
                       position: Vec3) {
        let (vertex_2d_entity, _, Some(edge_2d_entity), _) = self.new_local_vertex(
            commands,
            camera_manager,
            meshes,
            materials,
            Some(parent_vertex_2d_entity),
            position,
            Color::DARK_GRAY,
        ) else {
            panic!("No edges?");
        };
        commands.entity(vertex_2d_entity).insert(Compass);
        commands.entity(edge_2d_entity).insert(Compass);
    }

    fn new_local_vertex(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        parent_vertex_2d_entity_opt: Option<Entity>,
        position: Vec3,
        color: Color,
    ) -> (Entity, Entity, Option<Entity>, Option<Entity>) {
        let parent_vertex_3d_entity_opt =
            parent_vertex_2d_entity_opt.map(|parent_vertex_2d_entity| {
                *self
                    .vertex_entity_2d_to_3d(&parent_vertex_2d_entity)
                    .unwrap()
            });

        let mut vertex_3d_component = Vertex3d::new(0,0,0);
        vertex_3d_component.localize();
        vertex_3d_component.set_vec3(&position);
        let new_vertex_3d_entity = commands
            .spawn_empty()
            .insert(vertex_3d_component)
            .id();

        let (new_vertex_2d_entity, new_edge_2d_entity, new_edge_3d_entity) = vertex_3d_postprocess(
            commands,
            camera_manager,
            self,
            meshes,
            materials,
            parent_vertex_3d_entity_opt,
            new_vertex_3d_entity,
            None,
            color,
            false,
        );

        return (new_vertex_2d_entity, new_vertex_3d_entity, new_edge_2d_entity, new_edge_3d_entity);
    }

    fn compass_recalc(
        &mut self,
        camera_manager: &CameraManager,
        vertex_3d_q: &mut Query<(Entity, &mut Vertex3d)>,
        camera_transform: &Transform,
    ) {
        if let Ok((_, mut vertex_3d)) = vertex_3d_q.get_mut(self.compass_vertices[0]) {

            let right = camera_transform.right_direction();
            let up = right.cross(camera_transform.view_direction());

            let unit_length = 1.0 / camera_manager.camera_3d_scale();
            const COMPASS_POS: Vec2 = Vec2::new(530.0, 300.0);
            let offset_2d = camera_manager.camera_3d_offset().round() + Vec2::new(unit_length * -1.0 * COMPASS_POS.x, unit_length * COMPASS_POS.y);
            let offset_3d = (right * offset_2d.x) + (up * offset_2d.y);

            let vert_offset_3d = Vec3::ZERO + offset_3d;
            vertex_3d.set_vec3(&vert_offset_3d);

            let compass_length = unit_length * 25.0;
            let vert_offset_3d = Vec3::new(compass_length, 0.0, 0.0) + offset_3d;
            let (_, mut vertex_3d) = vertex_3d_q.get_mut(self.compass_vertices[1]).unwrap();
            vertex_3d.set_vec3(&vert_offset_3d);

            let vert_offset_3d = Vec3::new(0.0, compass_length, 0.0) + offset_3d;
            let (_, mut vertex_3d) = vertex_3d_q.get_mut(self.compass_vertices[2]).unwrap();
            vertex_3d.set_vec3(&vert_offset_3d);

            let vert_offset_3d = Vec3::new(0.0, 0.0, compass_length) + offset_3d;
            let (_, mut vertex_3d) = vertex_3d_q.get_mut(self.compass_vertices[3]).unwrap();
            vertex_3d.set_vec3(&vert_offset_3d);
        }
    }

    // returns true if vertex is owned by tab or unowned
    fn is_owned_by_tab_or_unowned(current_tab_id: TabId, owned_by_tab_q: &Query<&OwnedByTab>, entity: Entity) -> bool {
        if let Ok(owned_by_tab) = owned_by_tab_q.get(entity) {
            if *owned_by_tab.tab_id == current_tab_id {
                return true;
            }
        } else {
            return true;
        }
        return false;
    }

    // returns true if vertex is owned by tab
    fn is_owned_by_tab(current_tab_id: TabId, owned_by_tab_q: &Query<&OwnedByTab>, entity: Entity) -> bool {
        if let Ok(owned_by_tab) = owned_by_tab_q.get(entity) {
            if *owned_by_tab.tab_id == current_tab_id {
                return true;
            }
        }
        return false;
    }
}
