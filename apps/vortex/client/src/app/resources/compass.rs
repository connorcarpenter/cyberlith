use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Resource, SystemState},
    world::World,
};

use math::{Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::Transform,
    Assets,
};

use vortex_proto::components::Vertex3d;

use crate::app::resources::{
    camera_manager::CameraManager, canvas::Canvas, edge_manager::EdgeManager,
    tab_manager::TabManager, vertex_manager::VertexManager,
};

#[derive(Resource)]
pub struct Compass {
    resync: bool,
    compass_vertices_3d: Vec<Entity>,
}

impl Default for Compass {
    fn default() -> Self {
        Self {
            resync: false,
            compass_vertices_3d: Vec::new(),
        }
    }
}

impl Compass {
    pub fn queue_resync(&mut self) {
        self.resync = true;
    }

    pub fn sync_compass(
        &mut self,
        canvas: &Canvas,
        tab_manager: &TabManager,
        camera_manager: &CameraManager,
        vertex_3d_q: &mut Query<(Entity, &mut Vertex3d)>,
        transform_q: &Query<&Transform>,
    ) {
        if !self.resync {
            return;
        }

        self.resync = false;

        self.sync_compass_impl(
            canvas,
            tab_manager,
            camera_manager,
            vertex_3d_q,
            transform_q,
        );
    }

    pub fn vertices(&self) -> &Vec<Entity> {
        &self.compass_vertices_3d
    }

    fn sync_compass_impl(
        &self,
        canvas: &Canvas,
        tab_manager: &TabManager,
        camera_manager: &CameraManager,
        vertex_3d_q: &mut Query<(Entity, &mut Vertex3d)>,
        transform_q: &Query<&Transform>,
    ) {
        let Some(current_tab_state) = tab_manager.current_tab_state() else {
            return;
        };
        let camera_state = &current_tab_state.camera_state;
        let camera_3d_entity = camera_manager.camera_3d_entity().unwrap();

        let Ok(camera_transform) = transform_q.get(camera_3d_entity) else {
            return;
        };

        let Ok((_, mut vertex_3d)) = vertex_3d_q.get_mut(self.compass_vertices_3d[0]) else {
            return;
        };

        let right = camera_transform.right();
        let up = camera_transform.up();

        let unit_length = 1.0 / camera_state.camera_3d_scale();
        let compass_length = unit_length * 25.0;
        let mut compass_pos = canvas.texture_size() * 0.5;
        compass_pos.x -= 32.0;
        compass_pos.y -= 32.0;
        let offset_2d = camera_state.camera_3d_offset().round()
            + Vec2::new(
                unit_length * -1.0 * compass_pos.x,
                unit_length * compass_pos.y,
            );
        let offset_3d = (right * offset_2d.x) + (up * offset_2d.y);

        let vert_offset_3d = Vec3::ZERO + offset_3d;
        vertex_3d.set_vec3(&vert_offset_3d);

        let vert_offset_3d = Vec3::new(compass_length, 0.0, 0.0) + offset_3d;
        let (_, mut vertex_3d) = vertex_3d_q.get_mut(self.compass_vertices_3d[1]).unwrap();
        vertex_3d.set_vec3(&vert_offset_3d);

        let vert_offset_3d = Vec3::new(0.0, compass_length, 0.0) + offset_3d;
        let (_, mut vertex_3d) = vertex_3d_q.get_mut(self.compass_vertices_3d[2]).unwrap();
        vertex_3d.set_vec3(&vert_offset_3d);

        let vert_offset_3d = Vec3::new(0.0, 0.0, compass_length) + offset_3d;
        let (_, mut vertex_3d) = vertex_3d_q.get_mut(self.compass_vertices_3d[3]).unwrap();
        vertex_3d.set_vec3(&vert_offset_3d);
    }

    pub fn sync_compass_vertices(&self, world: &mut World) {
        let mut system_state: SystemState<Query<(&Vertex3d, &mut Transform)>> =
            SystemState::new(world);
        let mut vertex_3d_q = system_state.get_mut(world);

        for vertex_entity in self.compass_vertices_3d.iter() {
            let (vertex_3d, mut transform) = vertex_3d_q.get_mut(*vertex_entity).unwrap();
            transform.translation = vertex_3d.as_vec3();
        }
    }

    pub(crate) fn setup_compass(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
    ) {
        let (root_vertex_2d_entity, vertex_3d_entity, _, _) = vertex_manager.new_local_vertex(
            commands,
            camera_manager,
            edge_manager,
            meshes,
            materials,
            None,
            Vec3::ZERO,
            Color::WHITE,
            None,
        );
        self.compass_vertices_3d.push(vertex_3d_entity);

        self.new_compass_arm(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(100.0, 0.0, 0.0),
            Color::RED,
        );

        self.new_compass_arm(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(0.0, 100.0, 0.0),
            Color::GREEN,
        );

        self.new_compass_arm(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(0.0, 0.0, 100.0),
            Color::LIGHT_BLUE,
        );
    }

    fn new_compass_arm(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        root_vertex_2d_entity: Entity,
        position: Vec3,
        color: Color,
    ) {
        let (_, vertex_3d_entity, _, _) = vertex_manager.new_local_vertex(
            commands,
            camera_manager,
            edge_manager,
            meshes,
            materials,
            Some(root_vertex_2d_entity),
            position,
            color,
            None,
        );
        self.compass_vertices_3d.push(vertex_3d_entity);
    }
}
