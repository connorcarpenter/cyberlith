use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::Resource,
    query::With,
    system::{Commands, Query},
};
use bevy_log::{info, warn};
use naia_bevy_client::{Client, CommandsExt};

use input::{Input, Key, MouseButton};
use math::{convert_2d_to_3d, convert_3d_to_2d, Quat, Vec2, Vec3};
use render_api::{
    base::{CpuTexture2D},
    components::{
        Camera, CameraProjection, OrthographicProjection, Projection, RenderLayer, Transform,
        Viewport, Visibility,
    },
    shapes::{distance_to_2d_line, get_2d_line_transform_endpoint, set_2d_line_transform},
    Handle,
};
use vortex_proto::components::Vertex3d;

use crate::app::{
    components::{Edge2d, Edge3d, HoverCircle, SelectCircle, Vertex2d},
    set_3d_line_transform,
    resources::action_stack::{Action, ActionStack}
};

#[derive(Clone, Copy)]
pub enum ClickType {
    Left,
    Right,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum CanvasShape {
    Vertex,
    Edge,
    // Face,
}

#[derive(Resource)]
pub struct CanvasManager {
    is_visible: bool,
    next_visible: bool,
    is_2d: bool,

    canvas_texture: Option<Handle<CpuTexture2D>>,
    canvas_texture_size: Vec2,
    vertices_3d_to_2d: HashMap<Entity, Entity>,
    vertices_2d_to_3d: HashMap<Entity, Entity>,
    vertex_3d_entities_to_delete: Vec<Entity>,

    click_type: ClickType,
    click_start: Vec2,
    click_down: bool,

    pub camera_2d: Option<Entity>,
    pub layer_2d: RenderLayer,
    camera_2d_recalc: bool,

    pub camera_3d: Option<Entity>,
    pub layer_3d: RenderLayer,
    camera_3d_recalc: bool,
    camera_3d_offset: Vec2,
    camera_3d_rotation: Option<Quat>,
    camera_3d_scale: f32,

    pub hover_circle_entity: Option<Entity>,
    mouse_hover_recalc: bool,
    last_mouse_position: Vec2,
    hovered_entity: Option<Entity>,
    hover_type: CanvasShape,
    last_vertex_dragged: Option<(Entity, Vec3, Vec3)>,

    pub select_circle_entity: Option<Entity>,
    pub select_line_entity: Option<Entity>,
    selected_vertex: Option<Entity>,
    select_line_recalc: bool,
}

impl Default for CanvasManager {
    fn default() -> Self {
        Self {
            next_visible: false,
            is_visible: false,
            is_2d: true,

            canvas_texture: None,
            canvas_texture_size: Vec2::new(1280.0, 720.0),
            vertices_3d_to_2d: HashMap::new(),
            vertices_2d_to_3d: HashMap::new(),
            vertex_3d_entities_to_delete: Vec::new(),

            click_type: ClickType::Left,
            click_start: Vec2::ZERO,
            click_down: false,

            camera_2d: None,
            layer_2d: RenderLayer::default(),
            camera_2d_recalc: false,

            camera_3d: None,
            layer_3d: RenderLayer::default(),
            camera_3d_recalc: false,
            camera_3d_rotation: None,
            camera_3d_scale: 1.0,
            camera_3d_offset: Vec2::ZERO,

            hover_circle_entity: None,
            mouse_hover_recalc: false,
            last_mouse_position: Vec2::ZERO,
            hovered_entity: None,
            hover_type: CanvasShape::Vertex,
            last_vertex_dragged: None,

            select_circle_entity: None,
            select_line_entity: None,
            selected_vertex: None,
            select_line_recalc: false,
        }
    }
}

impl CanvasManager {
    pub fn update_input(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        input: &mut Input,
        action_stack: &mut ActionStack,
        transform_q: &mut Query<&mut Transform>,
        camera_q: &mut Query<(&mut Camera, &mut Projection)>,
        visibility_q: &mut Query<&mut Visibility>,
        vertex_3d_q: &mut Query<&mut Vertex3d>,
        vertex_2d_q: &Query<Entity, With<Vertex2d>>,
        edge_3d_q: &Query<(Entity, &Edge3d)>,
        edge_2d_q: &Query<(Entity, &Edge2d)>,
    ) {
        // Mouse wheel zoom..
        let scroll_y = input.consume_mouse_scroll();
        if scroll_y > 0.1 || scroll_y < -0.1 {
            self.camera_zoom(scroll_y);
        }

        // Mouse over
        if !self.click_down {
            self.update_mouse_hover(
                input.mouse_position(),
                transform_q,
                visibility_q,
                vertex_2d_q,
                edge_2d_q,
            );
        }
        self.update_select_line(input.mouse_position(), transform_q, visibility_q);

        // check keyboard input

        // (S)olid 3D View
        if input.is_pressed(Key::S) {
            // disable 2d camera, enable 3d camera
            self.set_3d_mode(camera_q);
        }
        // (W)ireframe 2D View
        else if input.is_pressed(Key::W) {
            // disable 3d camera, enable 2d camera
            self.set_2d_mode(camera_q);
        }
        // (G)ame Camera View
        else if input.is_pressed(Key::G) {
            self.set_camera_angle_ingame();
        }
        // Si(d)e Camera View
        else if input.is_pressed(Key::D) {
            self.set_camera_angle_side();
        }
        // (F)ront Camera View
        else if input.is_pressed(Key::F) {
            self.set_camera_angle_front();
        }
        // (T)op Camera View
        else if input.is_pressed(Key::T) {
            self.set_camera_angle_top();
        }
        // Delete
        else if input.is_pressed(Key::Delete) {
            self.handle_delete_key_press(commands, client, edge_3d_q);
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
                    commands,
                    client,
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

                if let Some((entity, old_pos, new_pos)) = self.last_vertex_dragged.take() {
                    action_stack.buffer_action(Action::MoveVertex(entity, old_pos, new_pos));
                }
            }
        }
    }

    pub fn update_3d_camera(&mut self, camera_q: &mut Query<(&mut Camera, &mut Transform)>) {
        if self.camera_3d_rotation.is_none() {
            let Some(camera_3d) = self.camera_3d else {
                return;
            };

            let Ok((_, transform)) = camera_q.get(camera_3d) else {
                return;
            };

            self.camera_3d_rotation = Some(transform.rotation.clone());
        }

        if self.camera_3d_recalc {
            self.camera_3d_recalc = false;
            self.camera_2d_recalc = true;

            let Some(camera_3d) = self.camera_3d else {
                return;
            };

            let Ok((_, mut camera_transform)) = camera_q.get_mut(camera_3d) else {
                return;
            };

            camera_transform.rotation = self.camera_3d_rotation.unwrap();
            camera_transform.scale = Vec3::splat(1.0 / self.camera_3d_scale);

            let right = camera_transform.right_direction();
            let up = right.cross(camera_transform.view_direction());

            camera_transform.translation = camera_transform.view_direction() * -100.0; // 100 units away from where looking
            let rounded_offset = self.camera_3d_offset.round();
            camera_transform.translation += right * rounded_offset.x;
            camera_transform.translation += up * rounded_offset.y;
        }
    }

    pub fn sync_vertices(
        &mut self,
        transform_q: &mut Query<&mut Transform>,
        camera_q: &Query<(&Camera, &Projection)>,
        vertex_3d_q: &Query<(Entity, &Vertex3d)>,
        edge_2d_q: &Query<(Entity, &Edge2d)>,
        edge_3d_q: &Query<(Entity, &Edge3d)>,
    ) {
        if !self.camera_2d_recalc {
            return;
        }
        self.camera_2d_recalc = false;

        self.mouse_hover_recalc = true;
        self.select_line_recalc = true;

        let Some(camera_3d) = self.camera_3d else {
            return;
        };

        let Ok(camera_transform) = transform_q.get(camera_3d) else {
            return;
        };
        let Ok((camera, camera_projection)) = camera_q.get(camera_3d) else {
            return;
        };

        let camera_viewport = camera.viewport.unwrap();
        let view_matrix = camera_transform.view_matrix();
        let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

        // update vertices
        for (vertex_3d_entity, vertex_3d) in vertex_3d_q.iter() {
            let Ok(mut vertex_3d_transform) = transform_q.get_mut(vertex_3d_entity) else {
                warn!("Vertex3d entity {:?} has no Transform", vertex_3d_entity);
                continue;
            };

            // 3d vertices
            vertex_3d_transform.translation.x = vertex_3d.x().into();
            vertex_3d_transform.translation.y = vertex_3d.y().into();
            vertex_3d_transform.translation.z = vertex_3d.z().into();

            let (coords, depth) = convert_3d_to_2d(
                &view_matrix,
                &projection_matrix,
                &camera_viewport.size_vec2(),
                &vertex_3d_transform.translation,
            );

            let Some(vertex_2d_entity) = self.vertex_entity_3d_to_2d(&vertex_3d_entity) else {
                panic!("Vertex3d entity {:?} has no corresponding Vertex2d entity", vertex_3d_entity);
            };
            let Ok(mut vertex_2d_transform) = transform_q.get_mut(*vertex_2d_entity) else {
                panic!("Vertex2d entity {:?} has no Transform", vertex_2d_entity);
            };

            // 2d vertices
            vertex_2d_transform.translation.x = coords.x;
            vertex_2d_transform.translation.y = coords.y;
            vertex_2d_transform.translation.z = depth;

            let scale_2d = self.camera_3d_scale * Vertex2d::RADIUS;
            vertex_2d_transform.scale = Vec3::splat(scale_2d);

            // update hover circle
            if let Some(hover_entity) = self.hovered_entity {
                if hover_entity == *vertex_2d_entity {
                    let hover_circle_entity = self.hover_circle_entity.unwrap();
                    let mut hover_circle_transform =
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

            let start_pos = transform_q
                .get(edge_endpoints.start)
                .unwrap()
                .translation
                .truncate();

            let end_pos = transform_q.get(*end_entity).unwrap().translation.truncate();
            let mut edge_transform = transform_q.get_mut(edge_entity).unwrap();
            set_2d_line_transform(&mut edge_transform, start_pos, end_pos);

            if let Some(hover_entity) = self.hovered_entity {
                if hover_entity == edge_entity {
                    edge_transform.scale.y = 3.0;
                }
            }
        }

        // update 3d edges
        for (edge_entity, edge_endpoints) in edge_3d_q.iter() {
            let start_pos = transform_q.get(edge_endpoints.start).unwrap().translation;

            let end_pos = transform_q.get(edge_endpoints.end).unwrap().translation;
            let mut edge_transform = transform_q.get_mut(edge_entity).unwrap();
            set_3d_line_transform(&mut edge_transform, start_pos, end_pos);
        }
    }

    pub fn update_visibility(&mut self, camera_q: &mut Query<(&mut Camera, &mut Transform)>) {
        if self.is_visible == self.next_visible {
            return;
        }
        self.is_visible = self.next_visible;

        let cameras_enabled = self.is_visible;

        if cameras_enabled {
            info!("Camera are ENABLED");
        } else {
            info!("Camera are DISABLED");
        }

        for (mut camera, _) in camera_q.iter_mut() {
            camera.is_active = cameras_enabled;
        }
    }

    pub fn poll_buffered_actions(
        &mut self,
        commands: &mut Commands,
        client: &Client,
        action_stack: &mut ActionStack,
    ) {
        if self.vertex_3d_entities_to_delete.len() == 0 {
            return;
        }

        let mut new_list = Vec::new();
        for vertex_3d_entity in self.vertex_3d_entities_to_delete.iter() {
            if commands
                .entity(*vertex_3d_entity)
                .authority(client)
                .unwrap()
                .is_granted()
            {
                // delete vertex
                action_stack.buffer_action(Action::DeleteVertex(*vertex_3d_entity));

                self.mouse_hover_recalc = true;
            } else {
                new_list.push(*vertex_3d_entity);
            }
        }
        self.vertex_3d_entities_to_delete = new_list;
    }

    pub fn update_camera_viewports(
        &mut self,
        texture_size: Vec2,
        camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
    ) {
        self.canvas_texture_size = texture_size;
        self.update_2d_camera_viewport(texture_size, camera_query);
        self.update_3d_camera_viewport(texture_size, camera_query);
    }

    pub fn canvas_texture(&self) -> Handle<CpuTexture2D> {
        self.canvas_texture.unwrap()
    }

    pub fn set_canvas_texture(&mut self, texture_size: Vec2, texture: Handle<CpuTexture2D>) {
        self.canvas_texture = Some(texture);
        self.canvas_texture_size = texture_size;
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn set_visibility(&mut self, visible: bool) {
        self.next_visible = visible;
    }

    pub fn recalculate_3d_view(&mut self) {
        self.camera_3d_recalc = true;
    }

    pub fn select_and_hover(&mut self, entity: &Entity) {
        self.selected_vertex = Some(*entity);
        self.hovered_entity = Some(*entity);

    }

    fn handle_delete_key_press(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        edge_3d_q: &Query<(Entity, &Edge3d)>,
    ) {
        if self.selected_vertex.is_none() {
            return;
        }

        // delete vertex

        let target_vertex_3d_entity = self
            .vertex_entity_2d_to_3d(&self.selected_vertex.unwrap())
            .unwrap();

        // make list of all children vertices
        let mut vertices_3d_to_delete = vec![];
        vertices_3d_to_delete_recurse(
            &mut vertices_3d_to_delete,
            target_vertex_3d_entity,
            edge_3d_q,
        );

        // check whether we can delete all vertices
        for vertex_3d_entity in vertices_3d_to_delete.iter() {
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
        }

        // delete them all!
        for vertex_3d_entity in vertices_3d_to_delete {
            let auth_status = commands.entity(vertex_3d_entity).authority(client).unwrap();
            if !auth_status.is_granted() {
                // request authority if needed
                commands.entity(vertex_3d_entity).request_authority(client);
            }

            self.mark_vertex_3d_entity_for_deletion(vertex_3d_entity);
        }

        // empty selection
        self.selected_vertex = None;
        self.select_line_recalc = true;
    }

    fn update_mouse_hover(
        &mut self,
        mouse_position: &Vec2,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &mut Query<&mut Visibility>,
        vertex_2d_q: &Query<Entity, With<Vertex2d>>,
        edge_2d_q: &Query<(Entity, &Edge2d)>,
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
        let mut hover_type = None;

        for vertex_entity in vertex_2d_q.iter() {
            let vertex_transform = transform_q.get(vertex_entity).unwrap();
            let vertex_position = vertex_transform.translation.truncate();
            let distance = vertex_position.distance(*mouse_position);
            if distance < least_distance {
                least_distance = distance;
                least_coords = vertex_position;
                least_entity = Some(vertex_entity);
                hover_type = Some(CanvasShape::Vertex);
            }
        }

        let mut is_hovering = least_distance <= (HoverCircle::DETECT_RADIUS * self.camera_3d_scale);

        // just setting edge thickness back to normal ... better way to do this?
        for (edge_entity, _) in edge_2d_q.iter() {
            let mut edge_transform = transform_q.get_mut(edge_entity).unwrap();
            edge_transform.scale.y = self.camera_3d_scale;
        }

        if !is_hovering {
            for (edge_entity, _) in edge_2d_q.iter() {
                let edge_transform = transform_q.get(edge_entity).unwrap();
                let edge_start = edge_transform.translation.truncate();
                let edge_end = get_2d_line_transform_endpoint(&edge_transform);

                let distance = distance_to_2d_line(*mouse_position, edge_start, edge_end);
                if distance < least_distance {
                    least_distance = distance;
                    least_entity = Some(edge_entity);
                    hover_type = Some(CanvasShape::Edge);
                }
            }

            is_hovering = least_distance <= (Edge2d::HOVER_THICKNESS * self.camera_3d_scale);
        }

        let hover_circle_entity = self.hover_circle_entity.unwrap();
        let Ok(mut hover_circle_visibility) = visibility_q.get_mut(hover_circle_entity) else {
            panic!("HoverCircle entity has no Transform or Visibility");
        };

        if is_hovering {
            self.hovered_entity = least_entity;
            self.hover_type = hover_type.unwrap();

            match hover_type {
                Some(CanvasShape::Vertex) => {
                    // hovering over vertex
                    let Ok(mut hover_circle_transform) = transform_q.get_mut(hover_circle_entity) else {
                        panic!("HoverCircle entity has no Transform");
                    };
                    hover_circle_transform.translation.x = least_coords.x;
                    hover_circle_transform.translation.y = least_coords.y;
                    hover_circle_transform.scale =
                        Vec3::splat(HoverCircle::DISPLAY_RADIUS * self.camera_3d_scale);

                    hover_circle_visibility.visible = true;
                }
                Some(CanvasShape::Edge) => {
                    // hovering over edge
                    let Ok(mut edge_transform) = transform_q.get_mut(least_entity.unwrap()) else {
                        panic!("Edge entity has no Transform");
                    };
                    edge_transform.scale.y = Edge2d::HOVER_THICKNESS * self.camera_3d_scale;

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

        if let Some(selected_vertex_entity) = self.selected_vertex {
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
                select_line_transform.scale.y = self.camera_3d_scale;
            }

            // sync select circle transform
            {
                let Ok(mut select_circle_transform) = transform_q.get_mut(select_circle_entity) else {
                    panic!("Select shape entities has no Transform");
                };

                select_circle_transform.translation = vertex_transform.translation;
                select_circle_transform.scale =
                    Vec3::splat(SelectCircle::RADIUS * self.camera_3d_scale);
            }

            select_shape_visibilities[0].visible = true;
            select_shape_visibilities[1].visible = true;
        } else {
            select_shape_visibilities[0].visible = false;
            select_shape_visibilities[1].visible = false;
        }

        //

        if let Some(selected_vertex_entity) = self.selected_vertex {
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
            select_line_transform.scale.y = self.camera_3d_scale;
        } else {
            let mut select_line_visibility = visibility_q.get_mut(select_line_entity).unwrap();
            select_line_visibility.visible = false;
        }
    }

    fn handle_mouse_click(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
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
                    let camera_3d = self.camera_3d.unwrap();
                    let camera_transform: Transform = *transform_q.get(camera_3d).unwrap();
                    let (camera, camera_projection) = camera_q.get(camera_3d).unwrap();

                    let camera_viewport = camera.viewport.unwrap();
                    let view_matrix = camera_transform.view_matrix();
                    let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

                    // get 2d vertex transform
                    let vertex_2d_entity = self.selected_vertex.unwrap();
                    let vertex_2d_transform = transform_q.get(vertex_2d_entity).unwrap();

                    // convert 2d to 3d
                    let new_3d_position = convert_2d_to_3d(
                        &view_matrix,
                        &projection_matrix,
                        &camera_viewport.size_vec2(),
                        &mouse_position,
                        vertex_2d_transform.translation.z,
                    );

                    // get 3d vertex transform
                    let vertex_3d_entity = self.vertex_entity_2d_to_3d(&vertex_2d_entity).unwrap();

                    // spawn new vertex
                    action_stack.buffer_action(Action::CreateVertex(*vertex_3d_entity, new_3d_position));

                    self.deselect_current_vertex(commands, client);
                }
                ClickType::Right => {
                    if self.selected_vertex.is_none() {
                        return;
                    }

                    // deselect vertex

                    self.deselect_current_vertex(commands, client);

                    self.camera_2d_recalc = true;
                }
            }
        } else {
            if cursor_is_hovering {
                match (self.hover_type, click_type) {
                    (CanvasShape::Vertex, ClickType::Left) => {
                        let Some(target_vertex_3d_entity) = self.vertex_entity_2d_to_3d(&self.hovered_entity.unwrap()) else {
                            panic!("Hovered entity does not have a 3d vertex! {:?}", self.hovered_entity.unwrap());
                        };

                        // select vertex

                        let auth_status = commands
                            .entity(*target_vertex_3d_entity)
                            .authority(client)
                            .unwrap();
                        if !auth_status.is_available() {
                            // do nothing, vertex is not available
                            info!(
                                "Vertex auth is not available for entity: {:?}. Current status: {:?}",
                                *target_vertex_3d_entity,
                                auth_status
                            );
                            return;
                        }
                        commands
                            .entity(*target_vertex_3d_entity)
                            .request_authority(client);

                        self.selected_vertex = self.hovered_entity;

                        self.camera_2d_recalc = true;
                    }
                    (CanvasShape::Vertex, ClickType::Right) => {
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

    fn mark_vertex_3d_entity_for_deletion(&mut self, vertex_3d_entity: Entity) {
        self.vertex_3d_entities_to_delete.push(vertex_3d_entity);
    }

    fn deselect_current_vertex(&mut self, commands: &mut Commands, client: &mut Client) {
        let vertex_3d_entity = self
            .vertex_entity_2d_to_3d(&self.selected_vertex.unwrap())
            .unwrap();
        commands.entity(*vertex_3d_entity).release_authority(client);
        self.selected_vertex = None;
    }

    fn handle_mouse_drag(
        &mut self,
        commands: &mut Commands,
        client: &Client,
        click_type: ClickType,
        mouse_position: Vec2,
        delta: Vec2,
        camera_q: &Query<(&mut Camera, &mut Projection)>,
        transform_q: &Query<&mut Transform>,
        vertex_3d_q: &mut Query<&mut Vertex3d>,
    ) {
        let vertex_is_selected = self.selected_vertex.is_some();

        if vertex_is_selected {
            match click_type {
                ClickType::Left => {
                    // move vertex
                    let vertex_2d_entity = self.selected_vertex.unwrap();
                    let vertex_3d_entity = self.vertex_entity_2d_to_3d(&vertex_2d_entity).unwrap();

                    let auth_status = commands
                        .entity(*vertex_3d_entity)
                        .authority(client)
                        .unwrap();
                    if !(auth_status.is_requested() || auth_status.is_granted()) {
                        // only continue to mutate if requested or granted authority over vertex
                        return;
                    }

                    // get camera
                    let camera_3d = self.camera_3d.unwrap();
                    let camera_transform: Transform = *transform_q.get(camera_3d).unwrap();
                    let (camera, camera_projection) = camera_q.get(camera_3d).unwrap();

                    let camera_viewport = camera.viewport.unwrap();
                    let view_matrix = camera_transform.view_matrix();
                    let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

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
                        self.last_vertex_dragged = Some((
                            *vertex_3d_entity,
                            old_3d_position,
                            new_3d_position,
                        ));
                    } else {
                        let old_3d_position = vertex_3d.as_vec3();
                        self.last_vertex_dragged = Some((*vertex_3d_entity, old_3d_position, new_3d_position));
                    }

                    vertex_3d.set_x(new_3d_position.x as i16);
                    vertex_3d.set_y(new_3d_position.y as i16);
                    vertex_3d.set_z(new_3d_position.z as i16);

                    // redraw
                    self.camera_2d_recalc = true;
                }
                ClickType::Right => {
                    // TODO: dunno if this is possible? shouldn't the vertex be deselected?
                }
            }
        } else {
            match click_type {
                ClickType::Left => {
                    self.camera_pan(delta);
                }
                ClickType::Right => {
                    self.camera_orbit(delta);
                }
            }
        }
    }

    fn set_2d_mode(&mut self, camera_query: &mut Query<(&mut Camera, &mut Projection)>) {
        if self.is_2d {
            return;
        }
        info!("Switched to Wireframe mode");
        self.is_2d = true;
        self.camera_2d_recalc = true;
        self.enable_cameras(camera_query, true);
    }

    fn set_3d_mode(&mut self, camera_query: &mut Query<(&mut Camera, &mut Projection)>) {
        if !self.is_2d {
            return;
        }
        info!("Switched to Solid mode");
        self.is_2d = false;
        self.enable_cameras(camera_query, false);
    }

    fn set_camera_angle_ingame(&mut self) {
        let mut rotation = Quat::from_rotation_y(f32::to_radians(0.0));

        // -60 seems to be 2:1 diablo isometric angle
        // -71.8 seems to be 3:2 warcraft angle
        // -64.849 seems to be the 7:4 angle we're looking for..
        rotation *= Quat::from_rotation_x(f32::to_radians(-64.849));

        self.set_camera_angle(rotation);
    }

    fn set_camera_angle_side(&mut self) {
        self.set_camera_angle(Quat::from_rotation_y(f32::to_radians(90.0)));
    }

    fn set_camera_angle_front(&mut self) {
        self.set_camera_angle(Quat::from_rotation_y(f32::to_radians(0.0)));
    }

    fn set_camera_angle_top(&mut self) {
        self.set_camera_angle(Quat::from_rotation_x(f32::to_radians(-90.0)));
    }

    fn camera_pan(&mut self, delta: Vec2) {
        self.camera_3d_offset += delta / self.camera_3d_scale;

        self.recalculate_3d_view();
    }

    fn camera_orbit(&mut self, delta: Vec2) {
        let Some(rotation) = self.camera_3d_rotation else {
            return;
        };

        let speed = -0.01;

        self.camera_3d_rotation = Some(
            rotation
                * Quat::from_rotation_y(delta.x * speed)
                * Quat::from_rotation_x(delta.y * speed),
        );

        self.recalculate_3d_view();
    }

    fn camera_zoom(&mut self, zoom_delta: f32) {
        let old_scale = self.camera_3d_scale;
        let new_scale = (old_scale + (zoom_delta * 0.01)).min(5.0).max(1.0);
        let scale_diff = new_scale - old_scale;
        self.camera_3d_scale = new_scale;

        if scale_diff.abs() > 0.0 {
            let old_screen_offset = self.camera_3d_offset * old_scale;
            let new_screen_offset = self.camera_3d_offset * new_scale;

            let offset_diff = new_screen_offset - old_screen_offset;

            self.camera_3d_offset -= offset_diff / new_scale;
        }

        self.recalculate_3d_view();
    }

    fn set_camera_angle(&mut self, angle: Quat) {
        self.camera_3d_rotation = Some(angle);
        self.camera_3d_offset = Vec2::ZERO;
        self.camera_3d_scale = 1.0;
        self.recalculate_3d_view();
    }

    fn enable_cameras(
        &self,
        camera_query: &mut Query<(&mut Camera, &mut Projection)>,
        enable_2d: bool,
    ) {
        let enable_3d = !enable_2d;

        if let Some(camera_2d) = self.camera_2d {
            if let Ok((mut camera, _)) = camera_query.get_mut(camera_2d) {
                camera.is_active = enable_2d;
            };
        }
        if let Some(camera_3d) = self.camera_3d {
            if let Ok((mut camera, _)) = camera_query.get_mut(camera_3d) {
                camera.is_active = enable_3d;
            };
        }
    }

    fn update_2d_camera_viewport(
        &self,
        texture_size: Vec2,
        camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
    ) {
        let Some(camera_entity) = self.camera_2d else {
            return;
        };
        let Ok((mut camera, mut transform, mut projection)) = camera_query.get_mut(camera_entity) else {
            return;
        };
        camera.viewport = Some(Viewport::new_at_origin(
            texture_size.x as u32,
            texture_size.y as u32,
        ));

        let center = texture_size * 0.5;

        *transform = Transform::from_xyz(center.x, center.y, 1.0)
            .looking_at(Vec3::new(center.x, center.y, 0.0), Vec3::NEG_Y);
        *projection =
            Projection::Orthographic(OrthographicProjection::new(texture_size.y, 0.0, 10.0));
    }

    fn update_3d_camera_viewport(
        &self,
        texture_size: Vec2,
        camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
    ) {
        let Some(camera_entity) = self.camera_3d else {
            return;
        };
        let Ok((mut camera, _, mut projection)) = camera_query.get_mut(camera_entity) else {
            return;
        };

        camera.viewport = Some(Viewport::new_at_origin(
            texture_size.x as u32,
            texture_size.y as u32,
        ));

        *projection =
            Projection::Orthographic(OrthographicProjection::new(texture_size.y, 0.0, 1000.0));
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

    pub fn cleanup_deleted_vertex(&mut self, entity_3d: &Entity, commands: &mut Commands, edge_2d_q: &Query<(Entity, &Edge2d)>, edge_3d_q: &Query<(Entity, &Edge3d)>) {
        // despawn 3d edge
        for (edge_3d_entity, edge_3d) in edge_3d_q.iter() {
            if edge_3d.start == *entity_3d {
                commands.entity(edge_3d_entity).despawn();
            }
        }

        if let Some(vertex_2d_entity) = self.unregister_3d_vertex(entity_3d) {
            // despawn 2d vertex
            commands.entity(vertex_2d_entity).despawn();

            // despawn 2d edge
            for (edge_2d_entity, edge_2d) in edge_2d_q.iter() {
                if edge_2d.start == vertex_2d_entity {
                    commands.entity(edge_2d_entity).despawn();
                }
            }
        } else {
            panic!("Vertex3d entity: `{:?}` has no corresponding Vertex2d entity", entity_3d);
        }
    }

    pub(crate) fn vertex_entity_3d_to_2d(&self, entity_3d: &Entity) -> Option<&Entity> {
        self.vertices_3d_to_2d.get(entity_3d)
    }

    fn vertex_entity_2d_to_3d(&self, entity_2d: &Entity) -> Option<&Entity> {
        self.vertices_2d_to_3d.get(entity_2d)
    }
}

fn vertices_3d_to_delete_recurse(
    list: &mut Vec<Entity>,
    parent_entity: &Entity,
    edge_3d_q: &Query<(Entity, &Edge3d)>,
) {
    info!("queuing {:?} for deletion", parent_entity);
    list.push(*parent_entity);

    for (_, edge_3d) in edge_3d_q.iter() {
        if edge_3d.end == *parent_entity {
            vertices_3d_to_delete_recurse(list, &edge_3d.start, edge_3d_q);
        }
    }
}
