use std::collections::HashMap;

use bevy_ecs::{entity::Entity, prelude::Resource, system::Query};
use bevy_log::info;

use math::{Vec2, Vec3};
use render_api::{base::CpuTexture2D, components::{Camera, OrthographicProjection, Projection, RenderLayer, Transform, Viewport}, Handle};

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
    vertices_3d_to_2d: HashMap<Entity, Entity>,

    pub click_type: ClickType,
    pub click_start: Vec2,
    pub click_down: bool,

    pub camera_2d: Option<Entity>,
    pub layer_2d: RenderLayer,

    pub camera_3d: Option<Entity>,
    pub layer_3d: RenderLayer,
    camera_3d_target: Vec3,
    camera_3d_target_distance: f32,
    camera_3d_recalc: bool,
}

impl Default for CanvasManager {
    fn default() -> Self {
        Self {
            next_visible: false,
            is_visible: false,
            is_2d: true,

            canvas_texture: None,
            vertices_3d_to_2d: HashMap::new(),

            click_type: ClickType::Left,
            click_start: Vec2::ZERO,
            click_down: false,

            camera_2d: None,
            layer_2d: RenderLayer::default(),

            camera_3d: None,
            layer_3d: RenderLayer::default(),
            camera_3d_target: Vec3::ZERO,
            camera_3d_target_distance: 100.0,
            camera_3d_recalc: false,
        }
    }
}

impl CanvasManager {
    pub fn camera_pan(&mut self, camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>, delta: Vec2) {
        let Some(camera_3d) = self.camera_3d else {
            return;
        };

        let (_, mut transform, _) = camera_query.get_mut(camera_3d).unwrap();

        let speed = 0.1;

        let right = transform.right_direction();
        let up = right.cross(transform.view_direction());

        self.camera_3d_target += right * delta.x * speed;
        self.camera_3d_target += up * delta.y * speed;

        self.camera_3d_recalc = true;
    }

    pub fn camera_orbit(&mut self, camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>, delta: Vec2) {
        let Some(camera_3d) = self.camera_3d else {
            return;
        };

        let (_, mut transform, _) = camera_query.get_mut(camera_3d).unwrap();

        let speed = 1.0;

        transform.orbit_rotate(delta * speed);

        self.camera_3d_recalc = true;
    }

    pub fn camera_zoom(&mut self, delta: f32) {
        self.camera_3d_target_distance = (self.camera_3d_target_distance + delta).min(200.0).max(10.0);

        self.camera_3d_recalc = true;
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

    pub fn update_3d_camera(&mut self, camera_q: &mut Query<(&mut Camera, &mut Transform)>) {
        if !self.camera_3d_recalc {
            return;
        }
        self.camera_3d_recalc = false;

        let Some(camera_3d) = self.camera_3d else {
            return;
        };

        let (_, mut transform) = camera_q.get_mut(camera_3d).unwrap();

        // keep Transform's rotation and scale the same, but base the position on self.camera_3d_target and self.camera_3d_target_distance
        transform.translation = self.camera_3d_target + transform.view_direction() * self.camera_3d_target_distance;
    }

    pub fn update_camera_viewports(
        &self,
        texture_size: Vec2,
        camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
    ) {
        self.update_2d_camera_viewport(texture_size, camera_query);
        self.update_3d_camera_viewport(texture_size, camera_query);
    }

    pub fn canvas_texture(&self) -> Handle<CpuTexture2D> {
        self.canvas_texture.unwrap()
    }

    pub fn set_canvas_texture(&mut self, texture: Handle<CpuTexture2D>) {
        self.canvas_texture = Some(texture);
    }

    pub fn is_visible(&self) -> bool {
        self.is_visible
    }

    pub fn set_visibility(&mut self, visible: bool) {
        self.next_visible = visible;
    }

    pub fn set_2d_mode(&mut self, camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>) {
        if self.is_2d {
            return;
        }
        info!("Switched to Wireframe mode");
        self.is_2d = true;
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
        *projection = Projection::Orthographic(OrthographicProjection {
            height: texture_size.y,
            near: 0.0,
            far: 10.0,
        });
    }

    fn update_3d_camera_viewport(
        &self,
        texture_size: Vec2,
        camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
    ) {
        let Some(camera_entity) = self.camera_3d else {
            return;
        };
        let Ok((mut camera, _, _)) = camera_query.get_mut(camera_entity) else {
            return;
        };
        camera.viewport = Some(Viewport::new_at_origin(
            texture_size.x as u32,
            texture_size.y as u32,
        ));
    }

    pub fn register_3d_vertex(&mut self, entity_3d: Entity, entity_2d: Entity) {
        self.vertices_3d_to_2d.insert(entity_3d, entity_2d);
    }

    pub fn unregister_3d_vertex(&mut self, entity_3d: &Entity) {
        self.vertices_3d_to_2d.remove(entity_3d);
    }

    pub fn vertex_entity_3d_to_2d(&self, entity_3d: &Entity) -> Option<&Entity> {
        self.vertices_3d_to_2d.get(entity_3d)
    }
}