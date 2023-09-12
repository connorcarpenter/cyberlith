
use bevy_ecs::{
    entity::Entity,
    system::Commands,
};

use math::Vec3;
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    Assets,
};

use crate::app::{
    components::LocalShape,
    resources::{
        camera_manager::CameraManager,
        shape_manager::ShapeManager
    },
};

pub struct Grid;

impl Grid {
    pub(crate) fn setup_grid(
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        shape_manager: &mut ShapeManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
    ) {
        Self::new_grid_corner(
            commands,
            camera_manager,
            shape_manager,
            meshes,
            materials,
            true,
            true,
            true,
        );
        Self::new_grid_corner(
            commands,
            camera_manager,
            shape_manager,
            meshes,
            materials,
            true,
            false,
            false,
        );
        Self::new_grid_corner(
            commands,
            camera_manager,
            shape_manager,
            meshes,
            materials,
            false,
            true,
            false,
        );
        Self::new_grid_corner(
            commands,
            camera_manager,
            shape_manager,
            meshes,
            materials,
            false,
            false,
            true,
        );
    }

    fn new_grid_corner(
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        shape_manager: &mut ShapeManager,
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

        let (root_vertex_2d_entity, root_vertex_3d_entity, _, _) = shape_manager.new_local_vertex(
            commands,
            camera_manager,
            meshes,
            materials,
            None,
            Vec3::new(grid_size * xf, (grid_size * yf) + grid_size, grid_size * zf),
            Color::DARK_GRAY,
        );
        commands.entity(root_vertex_2d_entity).insert(LocalShape);
        commands.entity(root_vertex_3d_entity).insert(LocalShape);

        Self::new_grid_vertex(
            commands,
            camera_manager,
            shape_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(
                neg_grid_size * xf,
                (grid_size * yf) + grid_size,
                grid_size * zf,
            ),
        );
        Self::new_grid_vertex(
            commands,
            camera_manager,
            shape_manager,
            meshes,
            materials,
            root_vertex_2d_entity,
            Vec3::new(
                grid_size * xf,
                (neg_grid_size * yf) + grid_size,
                grid_size * zf,
            ),
        );
        Self::new_grid_vertex(
            commands,
            camera_manager,
            shape_manager,
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
        commands: &mut Commands,
        camera_manager: &mut CameraManager,
        shape_manager: &mut ShapeManager,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        parent_vertex_2d_entity: Entity,
        position: Vec3,
    ) {
        let (vertex_2d_entity, vertex_3d_entity, Some(edge_2d_entity), Some(edge_3d_entity)) = shape_manager.new_local_vertex(
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
        commands.entity(vertex_2d_entity).insert(LocalShape);
        commands.entity(edge_2d_entity).insert(LocalShape);
        commands.entity(vertex_3d_entity).insert(LocalShape);
        commands.entity(edge_3d_entity).insert(LocalShape);
    }
}