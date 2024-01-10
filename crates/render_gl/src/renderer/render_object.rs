use std::collections::HashMap;

use gl::DrawArraysIndirectCommand;

use math::Mat4;
use render_api::{
    resources::MaterialOrSkinHandle,
    base::CpuMesh,
    components::{CameraProjection, Transform},
    Handle,
};

use crate::{core::{Cull, RenderStates, Context, GpuTexture2D}, renderer::{lights_shader_source, Light, RenderCamera}, GpuMaterialManager, GpuMeshManager, GpuSkinManager};

// Render Object
#[derive(Clone)]
pub struct RenderMeshes {
    mesh_handle_instance_map: HashMap<Handle<CpuMesh>, Vec<(MaterialOrSkinHandle, Mat4)>>,
}

impl RenderMeshes {
    pub fn new() -> Self {
        Self {
            mesh_handle_instance_map: HashMap::new(),
        }
    }

    pub fn add_instance(
        &mut self,
        mesh_handle: &Handle<CpuMesh>,
        mat_handle: &MaterialOrSkinHandle,
        transform: &Transform,
    ) {
        if !self.mesh_handle_instance_map.contains_key(mesh_handle) {
            self.mesh_handle_instance_map
                .insert(*mesh_handle, Vec::new());
        }
        let map = self.mesh_handle_instance_map.get_mut(mesh_handle).unwrap();
        map.push((*mat_handle, transform.compute_matrix()));
    }

    pub fn render<'a>(
        gpu_mesh_manager: &'a GpuMeshManager,
        gpu_material_manager: &'a GpuMaterialManager,
        gpu_skin_manager: &'a GpuSkinManager,
        render_camera: &'a RenderCamera<'a>,
        lights: &[&dyn Light],
        meshes: RenderMeshes,
    ) {
        if !gpu_skin_manager.is_ready() {
            return;
        }
        let (commands, instance_texture) =
            meshes.to_commands(gpu_mesh_manager, gpu_material_manager, gpu_skin_manager);

        let render_states = RenderStates {
            cull: Cull::Back,
            ..Default::default()
        };
        let fragment_shader = gpu_material_manager.fragment_shader();
        let vertex_shader_source = Self::vertex_shader_source(lights);
        Context::get()
            .program(vertex_shader_source, fragment_shader.source, |program| {
                gpu_material_manager.use_uniforms(program, render_camera, lights);
                gpu_skin_manager.use_uniforms(program);

                program.use_uniform(
                    "view_projection",
                    render_camera
                        .projection
                        .projection_matrix(&render_camera.camera.viewport_or_default())
                        * render_camera.transform.view_matrix(),
                );

                program.use_texture("instance_texture", &instance_texture);

                gpu_mesh_manager.draw(program, render_states, render_camera.camera, commands);
            })
            .expect("Failed compiling shader");
    }

    fn to_commands(
        self,
        gpu_mesh_manager: &GpuMeshManager,
        gpu_mat_manager: &GpuMaterialManager,
        gpu_skin_manager: &GpuSkinManager,
    ) -> (Vec<DrawArraysIndirectCommand>, GpuTexture2D) {
        let mut mesh_handle_transform_map = self.mesh_handle_instance_map;

        let mut commands = Vec::new();
        let mut instances_rows = Vec::new();
        let mut max_instance_count = 0;

        let mut mesh_handles = mesh_handle_transform_map
            .keys()
            .map(|handle| *handle)
            .collect::<Vec<_>>();
        mesh_handles.sort(); // TODO: is this still necessary??

        for mesh_handle in mesh_handles {
            let instances = mesh_handle_transform_map.remove(&mesh_handle).unwrap();
            let gpu_mesh = gpu_mesh_manager.get(&mesh_handle).unwrap();

            let count = gpu_mesh.count();
            let instance_count = instances.len();
            max_instance_count = max_instance_count.max(instance_count);
            let first = gpu_mesh.first();

            commands.push(DrawArraysIndirectCommand::new(
                count as u32,
                instance_count as u32,
                first as u32,
                0,
            ));

            let instance_row = Self::get_instance_row(gpu_mat_manager, gpu_skin_manager, instances);
            instances_rows.push(instance_row);
        }

        // convert instance rows to grid
        let mut instances_grid: Vec<[f32; 4]> = Vec::new();
        let instance_grid_width = max_instance_count * 4;
        for instance_row in instances_rows {
            let row_count = instance_row.len();
            let pad_count = instance_grid_width - row_count;
            for instance_cell in instance_row {
                instances_grid.push(instance_cell);
            }
            for _ in 0..pad_count {
                instances_grid.push([0.0; 4]);
            }
        }

        let texture_width = instance_grid_width as u32;
        let texture_height = commands.len() as u32;
        let mut instances_texture =
            GpuTexture2D::new_empty::<[f32; 4]>(texture_width, texture_height);
        instances_texture.fill_pure(&instances_grid);

        (commands, instances_texture)
    }

    fn get_instance_row(
        gpu_mat_manager: &GpuMaterialManager,
        gpu_skin_manager: &GpuSkinManager,
        instances: Vec<(MaterialOrSkinHandle, Mat4)>,
    ) -> Vec<[f32; 4]> {
        let mut instance_row = Vec::new();

        let indices = {
            // No need to order, just return the indices as is.
            (0..instances.len()).collect::<Vec<usize>>()
        };

        // Next, we can compute the instance buffers with that ordering.
        {
            for (mat_handle, transform) in indices.iter().map(|i| instances[*i]) {
                instance_row.push(transform.row(0).to_array());
                instance_row.push(transform.row(1).to_array());
                instance_row.push(transform.row(2).to_array());

                match mat_handle {
                    MaterialOrSkinHandle::Material(mat_handle) => {
                        let has_skin = -100.0;
                        let mat_index = gpu_mat_manager.get(&mat_handle).unwrap();
                        instance_row.push([has_skin, mat_index.index() as f32, 0.0, 0.0]);
                    }
                    MaterialOrSkinHandle::Skin(skin_handle) => {
                        let has_skin = 100.0;
                        let skin_index = gpu_skin_manager.get(&skin_handle).unwrap();
                        instance_row.push([has_skin, skin_index.index() as f32, 0.0, 0.0]);
                    }
                }
            }
        }

        instance_row
    }

    fn vertex_shader_source(lights: &[&dyn Light]) -> String {
        let mut output = lights_shader_source(lights);
        output.push_str(include_str!("../shaders/mesh.vert"));

        output
    }
}
