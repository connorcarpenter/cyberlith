
use bevy_ecs::component::Component;

use game_engine::{storage::{Handle, Storage}, render::{shapes::Cube, base::{CpuMaterial, Color, CpuMesh}}};

#[derive(Component, Clone)]
pub struct RenderHelper {
    pub(crate) cube_mesh_handle: Handle<CpuMesh>,

    pub(crate) red_mat_handle: Handle<CpuMaterial>,
    pub(crate) blue_mat_handle: Handle<CpuMaterial>,
    pub(crate) green_mat_handle: Handle<CpuMaterial>,
    pub(crate) yellow_mat_handle: Handle<CpuMaterial>,
    pub(crate) aqua_mat_handle: Handle<CpuMaterial>,
    pub(crate) pink_mat_handle: Handle<CpuMaterial>,
}

impl RenderHelper {
    pub fn new(
        meshes: &mut Storage<CpuMesh>,
        materials: &mut Storage<CpuMaterial>
    ) -> Self {

        let cube_mesh_handle = meshes.add(Cube);

        let red_mat_handle = materials.add(Color::RED);
        let blue_mat_handle = materials.add(Color::BLUE);
        let green_mat_handle = materials.add(Color::GREEN);
        let yellow_mat_handle = materials.add(Color::YELLOW);
        let aqua_mat_handle = materials.add(Color::AQUA);
        let pink_mat_handle = materials.add(Color::PINK);

        Self {
            cube_mesh_handle,
            red_mat_handle,
            blue_mat_handle,
            green_mat_handle,
            yellow_mat_handle,
            aqua_mat_handle,
            pink_mat_handle,
        }
    }
}
