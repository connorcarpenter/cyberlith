use std::collections::HashMap;

use bevy_ecs::{entity::Entity, prelude::Resource, query::{With, Without}, system::Query};
use bevy_log::info;

use math::{convert_3d_to_2d, Quat, Vec2, Vec3};
use render_api::{base::CpuTexture2D, components::{Camera, CameraProjection, OrthographicProjection, Projection, RenderLayer, Transform, Viewport}, Handle};
use vortex_proto::components::Vertex3d;

use crate::app::components::Vertex2d;

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

    pub click_type: ClickType,
    pub click_start: Vec2,
    pub click_down: bool,

    pub camera_2d: Option<Entity>,
    pub layer_2d: RenderLayer,
    camera_2d_recalc: bool,

    pub camera_3d: Option<Entity>,
    pub layer_3d: RenderLayer,
    camera_3d_recalc: bool,
    camera_3d_offset: Vec2,
    camera_3d_rotation: Option<Quat>,
    camera_3d_scale: f32,

    pub hover_entity: Option<Entity>,
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

            hover_entity: None,
        }
    }
}

impl CanvasManager {
    pub fn set_2d_mode(&mut self, camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>) {
        if self.is_2d {
            return;
        }
        info!("Switched to Wireframe mode");
        self.is_2d = true;
        self.camera_2d_recalc = true;
        self.enable_cameras(camera_query, true);
    }

    pub fn set_3d_mode(&mut self, camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>) {
        if !self.is_2d {
            return;
        }
        info!("Switched to Solid mode");
        self.is_2d = false;
        self.enable_cameras(camera_query, false);
    }

    fn set_camera_angle(&mut self, angle: Quat) {
        self.camera_3d_rotation = Some(angle);
        self.camera_3d_offset = Vec2::ZERO;
        self.camera_3d_scale = 1.0;
        self.camera_3d_recalc = true;
    }

    pub fn set_camera_angle_ingame(&mut self) {
        let mut rotation = Quat::from_rotation_y(f32::to_radians(0.0));

        // -60 seems to be 2:1 diablo isometric angle
        // -71.8 seems to be 3:2 warcraft angle
        // -64.849 seems to be the 7:4 angle we're looking for..
        rotation *= Quat::from_rotation_x(f32::to_radians(-64.849));

        self.set_camera_angle(rotation);
    }

    pub fn set_camera_angle_side(&mut self) {
        self.set_camera_angle(Quat::from_rotation_y(f32::to_radians(90.0)));
    }

    pub fn set_camera_angle_front(&mut self) {
        self.set_camera_angle(Quat::from_rotation_y(f32::to_radians(0.0)));
    }

    pub fn set_camera_angle_top(&mut self) {
        self.set_camera_angle(Quat::from_rotation_x(f32::to_radians(-90.0)));
    }

    pub fn camera_pan(&mut self, delta: Vec2) {
        self.camera_3d_offset += delta / self.camera_3d_scale;

        self.camera_3d_recalc = true;
    }

    pub fn camera_orbit(&mut self, delta: Vec2) {
        let Some(rotation) = self.camera_3d_rotation else {
            return;
        };

        let speed = -0.01;

        self.camera_3d_rotation = Some(rotation * Quat::from_rotation_y(delta.x * speed) * Quat::from_rotation_x(delta.y * speed));

        self.camera_3d_recalc = true;
    }

    pub fn camera_zoom(&mut self, zoom_delta: f32) {
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
        camera_q: &Query<(&Camera, &Transform, &Projection), (Without<Vertex3d>, Without<Vertex2d>)>,
        vertex_3d_q: &mut Query<(Entity, &Vertex3d, &mut Transform), Without<Vertex2d>>,
        vertex_2d_q: &mut Query<&mut Transform, With<Vertex2d>>,
    ) {
        if !self.camera_2d_recalc {
            return;
        }
        self.camera_2d_recalc = false;

        let Some(camera_3d) = self.camera_3d else {
            return;
        };

        let Ok((camera, camera_transform, camera_projection)) = camera_q.get(camera_3d) else {
            return;
        };

        let camera_viewport = camera.viewport.unwrap();
        let camera_viewport_size = Vec2::new(camera_viewport.width as f32, camera_viewport.height as f32);
        let view_matrix = camera_transform.view_matrix();
        let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

        for (vertex_3d_entity, vertex_3d, mut vertex_3d_transform) in vertex_3d_q.iter_mut() {
            vertex_3d_transform.translation.x = vertex_3d.x().into();
            vertex_3d_transform.translation.y = vertex_3d.y().into();
            vertex_3d_transform.translation.z = vertex_3d.z().into();

            let Some(vertex_2d_entity) = self.vertex_entity_3d_to_2d(&vertex_3d_entity) else {
                panic!("Vertex3d entity {:?} has no corresponding Vertex2d entity", vertex_3d_entity);
            };
            let Ok(mut vertex_2d_transform) = vertex_2d_q.get_mut(*vertex_2d_entity) else {
                info!("Vertex2d entity {:?} has no Transform", vertex_2d_entity);
                continue;
            };

            let (coords, depth) = convert_3d_to_2d(
                &view_matrix,
                &projection_matrix,
                &camera_viewport_size,
                &vertex_3d_transform.translation,
            );
            vertex_2d_transform.translation.x = coords.x;
            vertex_2d_transform.translation.y = coords.y;
            vertex_2d_transform.translation.z = depth;

            let scale_2d = self.camera_3d_scale * Vertex2d::RADIUS;
            vertex_2d_transform.scale = Vec3::splat(scale_2d);
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

    fn enable_cameras(
        &self,
        camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
        enable_2d: bool,
    ) {
        let enable_3d = !enable_2d;

        if let Some(camera_2d) = self.camera_2d {
            if let Ok((mut camera, _, _)) = camera_query.get_mut(camera_2d) {
                camera.is_active = enable_2d;
            };
        }
        if let Some(camera_3d) = self.camera_3d {
            if let Ok((mut camera, _, _)) = camera_query.get_mut(camera_3d) {
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