use std::collections::HashMap;

use bevy_ecs::{change_detection::ResMut, entity::Entity, prelude::Resource, query::With, system::Query};
use bevy_log::{info, warn};

use input::{Input, Key, MouseButton};
use math::{convert_3d_to_2d, Quat, Vec2, Vec3};
use render_api::{base::CpuTexture2D, components::{Camera, CameraProjection, OrthographicProjection, Projection, RenderLayer, Transform, Viewport, Visibility}, Handle, shapes::set_line_transform};
use vortex_proto::components::Vertex3d;

use crate::app::components::{HoverCircle, LineEntities, SelectCircle, Vertex2d};

#[derive(Clone, Copy)]
pub enum ClickType {
    Left,
    Right,
}

#[derive(Resource)]
pub struct CanvasManager {
    is_visible: bool,
    next_visible: bool,
    is_2d: bool,

    canvas_texture: Option<Handle<CpuTexture2D>>,
    canvas_texture_size: Vec2,
    vertices_3d_to_2d: HashMap<Entity, Entity>,

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
    hovered_vertex: Option<Entity>,

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
            hovered_vertex: None,

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
        input: &mut ResMut<Input>,
        transform_q: &mut Query<&mut Transform>,
        camera_q: &mut Query<(&mut Camera, &mut Projection)>,
        visibility_q: &mut Query<&mut Visibility>,
        vertex_2d_q: &Query<Entity, With<Vertex2d>>,
    ) {
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

        // Mouse wheel zoom..
        let scroll_y = input.consume_mouse_scroll();
        if scroll_y > 0.1 || scroll_y < -0.1 {
            self.camera_zoom(scroll_y);
        }

        // Mouse over
        self.update_mouse_hover(input.mouse_position(), transform_q, visibility_q, vertex_2d_q);
        self.update_select_line(input.mouse_position(), transform_q);

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
                    self.handle_mouse_drag(self.click_type, delta);
                }
            } else {
                // haven't clicked yet
                self.click_down = true;
                self.click_start = *input.mouse_position();
                self.handle_mouse_click(self.click_type);
            }
        } else {
            if self.click_down {
                // release click
                self.click_down = false;
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

            let Ok((_, mut transform)) = camera_q.get_mut(camera_3d) else {
                return;
            };

            // keep Transform's rotation and scale the same, but base the position on self.camera_3d_target and self.camera_3d_target_distance
            transform.rotation = self.camera_3d_rotation.unwrap();
            transform.scale = Vec3::splat(1.0 / self.camera_3d_scale);

            let right = transform.right_direction();
            let up = right.cross(transform.view_direction());

            transform.translation = transform.view_direction() * -100.0; // 100 units away from where looking
            transform.translation += right * self.camera_3d_offset.x;
            transform.translation += up * self.camera_3d_offset.y;
        }
    }

    pub fn sync_vertices(
        &mut self,
        transform_q: &mut Query<&mut Transform>,
        camera_q: &Query<(&Camera, &Projection)>,
        vertex_3d_q: &Query<(Entity, &Vertex3d)>,
        visibility_q: &mut Query<&mut Visibility>,
        edge_q: &Query<(Entity, &LineEntities)>,
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
        let camera_viewport_size = Vec2::new(camera_viewport.width as f32, camera_viewport.height as f32);
        let view_matrix = camera_transform.view_matrix();
        let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

        // update verticies
        for (vertex_3d_entity, vertex_3d) in vertex_3d_q.iter() {
            let Ok(mut vertex_3d_transform) = transform_q.get_mut(vertex_3d_entity) else {
                warn!("Vertex3d entity {:?} has no Transform", vertex_3d_entity);
                continue;
            };

            vertex_3d_transform.translation.x = vertex_3d.x().into();
            vertex_3d_transform.translation.y = vertex_3d.y().into();
            vertex_3d_transform.translation.z = vertex_3d.z().into();

            let (coords, depth) = convert_3d_to_2d(
                &view_matrix,
                &projection_matrix,
                &camera_viewport_size,
                &vertex_3d_transform.translation,
            );

            let Some(vertex_2d_entity) = self.vertex_entity_3d_to_2d(&vertex_3d_entity) else {
                panic!("Vertex3d entity {:?} has no corresponding Vertex2d entity", vertex_3d_entity);
            };
            let Ok(mut vertex_2d_transform) = transform_q.get_mut(*vertex_2d_entity) else {
                panic!("Vertex2d entity {:?} has no Transform", vertex_2d_entity);
            };

            vertex_2d_transform.translation.x = coords.x;
            vertex_2d_transform.translation.y = coords.y;
            vertex_2d_transform.translation.z = depth;

            let scale_2d = self.camera_3d_scale * Vertex2d::RADIUS;
            vertex_2d_transform.scale = Vec3::splat(scale_2d);
        }

        // update edges
        for (edge_entity, edge_endpoints) in edge_q.iter() {
            let Some(end_entity) = self.vertex_entity_3d_to_2d(&edge_endpoints.end_3d) else {
                warn!("Edge entity {:?} has no endpoint entity", edge_entity);
                continue;
            };

            let start_pos = transform_q.get(edge_endpoints.start).unwrap().translation.truncate();

            let end_pos = transform_q.get(*end_entity).unwrap().translation.truncate();
            let mut edge_transform = transform_q.get_mut(edge_entity).unwrap();
            set_line_transform(&mut edge_transform, &start_pos, &end_pos);
        }

        // update selected vertex circle & line

        let Ok(mut select_shape_visibilities) = visibility_q.get_many_mut([self.select_circle_entity.unwrap(), self.select_line_entity.unwrap()]) else {
            panic!("Select shape entities has no Visibility");
        };

        if let Some(selected_vertex_entity) = self.selected_vertex {
            let vertex_transform = {
                let Ok(vertex_transform) = transform_q.get(selected_vertex_entity) else {
                    return;
                };
                *vertex_transform
            };

            let Ok(mut select_circle_transform) = transform_q.get_mut(self.select_circle_entity.unwrap()) else {
                panic!("Select shape entities has no Transform");
            };

            select_circle_transform.translation = vertex_transform.translation;
            select_circle_transform.scale = Vec3::splat(SelectCircle::RADIUS * self.camera_3d_scale);

            select_shape_visibilities[0].visible = true;
            select_shape_visibilities[1].visible = true;
        } else {
            select_shape_visibilities[0].visible = false;
            select_shape_visibilities[1].visible = false;
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

    fn update_mouse_hover(
        &mut self,
        mouse_position: &Vec2,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &mut Query<&mut Visibility>,
        vertex_2d_q: &Query<Entity, With<Vertex2d>>,
    ) {
        if mouse_position.x as i16 != self.last_mouse_position.x as i16 || mouse_position.y as i16 != self.last_mouse_position.y as i16 {
            // mouse moved!
            self.mouse_hover_recalc = true;
            self.select_line_recalc = true;
            self.last_mouse_position = *mouse_position;
        }

        if !self.mouse_hover_recalc {
            return;
        }

        self.mouse_hover_recalc = false;

        let radius = HoverCircle::RADIUS * self.camera_3d_scale;
        let mut least_distance = f32::MAX;
        let mut least_coords = Vec2::ZERO;
        let mut least_entity = None;
        for vertex_entity in vertex_2d_q.iter() {
            let vertex_transform = transform_q.get(vertex_entity).unwrap();
            let vertex_position = vertex_transform.translation.truncate();
            let distance = vertex_position.distance(*mouse_position);
            if distance < least_distance {
                least_distance = distance;
                least_coords = vertex_position;
                least_entity = Some(vertex_entity);
            }
        }

        let is_hovered = least_distance <= radius;

        let hover_circle_entity = self.hover_circle_entity.unwrap();
        let Ok(mut hover_transform) = transform_q.get_mut(hover_circle_entity) else {
            panic!("HoverCircle entity has no Transform or Visibility");
        };
        let Ok(mut hover_visibility) = visibility_q.get_mut(hover_circle_entity) else {
            panic!("HoverCircle entity has no Transform or Visibility");
        };

        if is_hovered {
            self.hovered_vertex = least_entity;

            hover_transform.translation.x = least_coords.x;
            hover_transform.translation.y = least_coords.y;
            hover_visibility.visible = true;
        } else {
            self.hovered_vertex = None;
            hover_visibility.visible = false;
        }

        hover_transform.scale = Vec3::splat(radius);
    }

    fn update_select_line(&mut self, mouse_position: &Vec2, transform_q: &mut Query<&mut Transform>) {
        // update selected vertex circle & line

        if let Some(selected_vertex_entity) = self.selected_vertex {
            let select_line_entity = self.select_line_entity.unwrap();

            let vertex_transform = {
                let Ok(vertex_transform) = transform_q.get(selected_vertex_entity) else {
                    return;
                };
                *vertex_transform
            };

            let Ok(mut select_line_transform) = transform_q.get_mut(select_line_entity) else {
                panic!("Select line entity has no Transform");
            };

            set_line_transform(&mut select_line_transform, &vertex_transform.translation.truncate(), mouse_position);
        }
    }

    fn handle_mouse_drag(&mut self, click_type: ClickType, delta: Vec2) {
        let vertex_is_selected = self.selected_vertex.is_some();
        let cursor_is_hovering = self.hovered_vertex.is_some();

        if vertex_is_selected || cursor_is_hovering {
            // TODO: move vertex?
            return;
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

    fn handle_mouse_click(
        &mut self,
        click_type: ClickType,
    ) {
        // let vertex_is_selected = self.selected_vertex.is_some();
        let cursor_is_hovering = self.hovered_vertex.is_some();

        if cursor_is_hovering {
            match click_type {
                ClickType::Left => {
                    // select vertex

                    if self.hovered_vertex == self.selected_vertex {
                        // do nothing, already selected
                        return;
                    }

                    self.selected_vertex = self.hovered_vertex;

                    self.camera_2d_recalc = true;
                }
                ClickType::Right => {
                    // TODO: delete vertex?
                }
            }
        } else {
            match click_type {
                ClickType::Left => {
                    // TODO: create new vertex
                }
                ClickType::Right => {
                    // deselect vertex
                    if self.selected_vertex.is_some() {
                        self.selected_vertex = None;
                        self.camera_2d_recalc = true;
                    }
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

        self.camera_3d_recalc = true;
    }

    fn camera_orbit(&mut self, delta: Vec2) {
        let Some(rotation) = self.camera_3d_rotation else {
            return;
        };

        let speed = -0.01;

        self.camera_3d_rotation = Some(rotation * Quat::from_rotation_y(delta.x * speed) * Quat::from_rotation_x(delta.y * speed));

        self.camera_3d_recalc = true;
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

        self.camera_3d_recalc = true;
    }

    fn set_camera_angle(&mut self, angle: Quat) {
        self.camera_3d_rotation = Some(angle);
        self.camera_3d_offset = Vec2::ZERO;
        self.camera_3d_scale = 1.0;
        self.camera_3d_recalc = true;
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

        *transform = Transform::from_xyz(center.x, center.y, -1.0)
            .looking_at(Vec3::new(center.x, center.y, 0.0), Vec3::NEG_Y);
        *projection = Projection::Orthographic(OrthographicProjection::new(
            texture_size.y,
            0.0,
            10.0,
        ));
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

        *projection = Projection::Orthographic(OrthographicProjection::new(
            texture_size.y,
            0.0,
            1000.0,
        ));
    }

    pub fn register_3d_vertex(&mut self, entity_3d: Entity, entity_2d: Entity) {
        self.vertices_3d_to_2d.insert(entity_3d, entity_2d);
        self.camera_2d_recalc = true;
    }

    pub fn unregister_3d_vertex(&mut self, entity_3d: &Entity) -> Option<Entity> {
        self.vertices_3d_to_2d.remove(entity_3d)
    }

    pub fn vertex_entity_3d_to_2d(&self, entity_3d: &Entity) -> Option<&Entity> {
        self.vertices_3d_to_2d.get(entity_3d)
    }
}