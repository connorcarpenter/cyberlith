use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Resource, SystemState},
    world::World,
};

use math::Vec3;
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::Transform,
    Assets,
};
use vortex_proto::components::Vertex3d;

use crate::app::resources::{
    camera_manager::CameraManager, edge_manager::EdgeManager, vertex_manager::VertexManager,
};

#[derive(Resource)]
pub struct Grid {
    resync: bool,
    grid_vertices_3d: Vec<Entity>,
}

impl Default for Grid {
    fn default() -> Self {
        Self {
            resync: false,
            grid_vertices_3d: Vec::new(),
        }
    }
}

impl Grid {
    pub fn queue_resync(&mut self) {
        self.resync = true;
    }

    pub fn sync_grid_vertices(&mut self, world: &mut World) {
        if !self.resync {
            return;
        }

        self.resync = false;

        let mut system_state: SystemState<Query<(&Vertex3d, &mut Transform)>> =
            SystemState::new(world);
        let mut vertex_3d_q = system_state.get_mut(world);

        for vertex_entity in self.grid_vertices_3d.iter() {
            let (vertex_3d, mut transform) = vertex_3d_q.get_mut(*vertex_entity).unwrap();
            transform.translation = vertex_3d.as_vec3();
        }
    }

    pub fn vertices(&self) -> &Vec<Entity> {
        &self.grid_vertices_3d
    }

    pub(crate) fn setup_grid(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
    ) {
        self.new_grid_corner(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            meshes,
            materials,
            true,
            true,
            true,
        );
        self.new_grid_corner(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            meshes,
            materials,
            true,
            false,
            false,
        );
        self.new_grid_corner(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            meshes,
            materials,
            false,
            true,
            false,
        );
        self.new_grid_corner(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            meshes,
            materials,
            false,
            false,
            true,
        );
    }

    fn new_grid_corner(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        x: bool,
        y: bool,
        z: bool,
    ) {
        let xf = if x { 1.0 } else { -1.0 };
        let yf = if y { 1.0 } else { -1.0 };
        let zf = if z { 1.0 } else { -1.0 };

        let grid_size: f32 = 100.0;
        let neg_grid_size: f32 = -grid_size;

        let (root_vertex_2d_entity, root_vertex_3d_entity, _, _) = vertex_manager.new_local_vertex(
            commands,
            camera_manager,
            edge_manager,
            meshes,
            materials,
            None,
            Vec3::new(grid_size * xf, (grid_size * yf) + grid_size, grid_size * zf),
            Color::GRAY,
            None,
        );
        self.grid_vertices_3d.push(root_vertex_3d_entity);

        self.new_grid_vertex(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(
                neg_grid_size * xf,
                (grid_size * yf) + grid_size,
                grid_size * zf,
            ),
        );
        self.new_grid_vertex(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(
                grid_size * xf,
                (neg_grid_size * yf) + grid_size,
                grid_size * zf,
            ),
        );
        self.new_grid_vertex(
            commands,
            camera_manager,
            vertex_manager,
            edge_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(
                grid_size * xf,
                (grid_size * yf) + grid_size,
                neg_grid_size * zf,
            ),
        );
    }

    fn new_grid_vertex(
        &mut self,
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        edge_manager: &mut EdgeManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        parent_vertex_2d_entity: Entity,
        position: Vec3,
    ) {
        let (_, vertex_3d_entity, _, _) = vertex_manager.new_local_vertex(
            commands,
            camera_manager,
            edge_manager,
            meshes,
            materials,
            Some(parent_vertex_2d_entity),
            position,
            Color::GRAY,
            None,
        );
        self.grid_vertices_3d.push(vertex_3d_entity);
    }
}
