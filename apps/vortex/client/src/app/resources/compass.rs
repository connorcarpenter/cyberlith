use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Resource},
};

use math::{Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::Transform,
    Assets,
};

use vortex_proto::components::Vertex3d;

use crate::app::{
    components::LocalShape,
    resources::{
        camera_manager::CameraManager, camera_state::CameraState, edge_manager::EdgeManager,
        face_manager::FaceManager, vertex_manager::VertexManager,
    },
};

#[derive(Resource)]
pub struct Compass {
    resync: bool,
    compass_vertices: Vec<Entity>,
}

impl Default for Compass {
    fn default() -> Self {
        Self {
            resync: false,
            compass_vertices: Vec::new(),
        }
    }
}

impl Compass {
    pub fn queue_resync(&mut self) {
        self.resync = true;
    }

    pub(crate) fn setup_compass(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        face_manager: &mut FaceManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
    ) {
        let (root_vertex_2d_entity, vertex_3d_entity, _, _) = vertex_manager.new_local_vertex(
            commands,
            camera_manager,
            edge_manager,
            face_manager,
            meshes,
            materials,
            None,
            Vec3::ZERO,
            Color::WHITE,
        );
        self.compass_vertices.push(vertex_3d_entity);
        commands.entity(root_vertex_2d_entity).insert(LocalShape);
        commands.entity(vertex_3d_entity).insert(LocalShape);

        self.new_compass_arm(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            face_manager,
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
            face_manager,
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
            face_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(0.0, 0.0, 100.0),
            Color::LIGHT_BLUE,
        );
    }

    pub fn sync_compass(
        &mut self,
        camera_3d_entity: &Entity,
        camera_state: &CameraState,
        vertex_3d_q: &mut Query<(Entity, &mut Vertex3d)>,
        transform_q: &Query<&Transform>,
    ) {
        if !self.resync {
            return;
        }

        self.resync = false;

        let Ok(camera_transform) = transform_q.get(*camera_3d_entity) else {
            return;
        };

        let Ok((_, mut vertex_3d)) = vertex_3d_q.get_mut(self.compass_vertices[0]) else {
            return;
        };

        let right = camera_transform.right_direction();
        let up = right.cross(camera_transform.view_direction());

        let unit_length = 1.0 / camera_state.camera_3d_scale();
        const COMPASS_POS: Vec2 = Vec2::new(530.0, 300.0);
        let offset_2d = camera_state.camera_3d_offset().round()
            + Vec2::new(
                unit_length * -1.0 * COMPASS_POS.x,
                unit_length * COMPASS_POS.y,
            );
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

    fn new_compass_arm(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        face_manager: &mut FaceManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        root_vertex_2d_entity: Entity,
        position: Vec3,
        color: Color,
    ) {
        let (vertex_2d_entity, vertex_3d_entity, Some(edge_2d_entity), Some(edge_3d_entity)) = vertex_manager.new_local_vertex(
            commands,
            camera_manager,
            edge_manager,
            face_manager,
            meshes,
            materials,
            Some(root_vertex_2d_entity),
            position,
            color,
        ) else {
            panic!("No edges?");
        };
        self.compass_vertices.push(vertex_3d_entity);
        commands.entity(vertex_2d_entity).insert(LocalShape);
        commands.entity(vertex_3d_entity).insert(LocalShape);
        commands.entity(edge_2d_entity).insert(LocalShape);
        commands.entity(edge_3d_entity).insert(LocalShape);
    }
}
