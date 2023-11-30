use std::collections::HashMap;

use gl::DrawArraysIndirectCommand;

use math::Mat4;
use render_api::{base::AxisAlignedBoundingBox, components::{CameraProjection, Transform}, Handle};
use render_api::base::{CpuMaterial, CpuMesh};
use render_api::components::Camera;

use crate::{AssetMapping, core::{Context, InstanceBuffer, Program, RenderStates}, GpuMaterial, GpuMaterialManager, GpuMesh, GpuMeshManager, renderer::{Instances, lights_shader_source, Light, Material, RenderCamera}};
use crate::core::GpuTexture2D;

// Render Object
#[derive(Clone)]
pub struct RenderObject {
    mesh_handle_transform_map: HashMap<Handle<CpuMesh>, Vec<(Handle<CpuMaterial>, Mat4)>>,
}

impl RenderObject{
    pub fn new() -> Self {
        Self {
            mesh_handle_transform_map: HashMap::new(),
        }
    }

    pub fn add_transform(&mut self, mesh_handle: &Handle<CpuMesh>, mat_handle: &Handle<CpuMaterial>, transform: &Transform) {
        if !self.mesh_handle_transform_map.contains_key(mesh_handle) {
            self.mesh_handle_transform_map.insert(*mesh_handle, Vec::new());
        }
        let map = self.mesh_handle_transform_map.get_mut(mesh_handle).unwrap();
        map.push((*mat_handle, transform.compute_matrix()));
    }

    pub fn to_commands(self, gpu_mesh_manager: &GpuMeshManager, gpu_mat_manager: &GpuMaterialManager) -> (Vec<DrawArraysIndirectCommand>, GpuTexture2D, usize) {

        let mut mesh_handle_transform_map = self.mesh_handle_transform_map;

        let mut commands = Vec::new();
        let mut instances_rows = Vec::new();
        let mut max_instance_count = 0;

        let mut mesh_handles = mesh_handle_transform_map.keys().map(|handle| *handle).collect::<Vec<_>>();
        mesh_handles.sort();

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

            let instance_row = Self::get_instance_row(gpu_mat_manager, instances);
            instances_rows.push(instance_row);
        }

        // convert transform rows to grid
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
        let texture_height = (commands.len() as u32);
        let mut instances_texture = GpuTexture2D::new_empty::<[f32; 4]>(texture_width, texture_height);
        instances_texture.fill_pure(&instances_grid);

        (commands, instances_texture, max_instance_count)
    }


    fn get_instance_row(gpu_mat_manager: &GpuMaterialManager, instances: Vec<(Handle<CpuMaterial>, Mat4)>) -> Vec<[f32; 4]> {

        let mut instance_row = Vec::new();

        let indices = {
            // No need to order, just return the indices as is.
            (0..instances.len()).collect::<Vec<usize>>()
        };

        // Next, we can compute the instance buffers with that ordering.
        {
            for (mat_handle, transformation) in indices.iter().map(|i| instances[*i]) {

                instance_row.push(transformation.row(0).to_array());
                instance_row.push(transformation.row(1).to_array());
                instance_row.push(transformation.row(2).to_array());


                let mat_index = gpu_mat_manager.get(&mat_handle).unwrap();
                instance_row.push([mat_index.index() as f32, 0.0, 0.0, 0.0]);
            }
        }

        instance_row
    }
}

// Instanced rendering
pub struct RenderObjectInstanced;

impl RenderObjectInstanced {
    pub fn render<'a>(
        gpu_mesh_manager: &'a GpuMeshManager,
        gpu_material_manager: &'a GpuMaterialManager,
        render_camera: &'a RenderCamera<'a>,
        lights: &[&dyn Light],
        object: RenderObject,
    ) {
        let (commands, instance_texture, max_instances) = object.to_commands(gpu_mesh_manager, gpu_material_manager);

        let render_states = gpu_material_manager.render_states();
        let fragment_shader = gpu_material_manager.fragment_shader();
        let vertex_shader_source = Self::vertex_shader_source(lights);
        Context::get()
            .program(vertex_shader_source, fragment_shader.source, |program| {
                gpu_material_manager.use_uniforms(program, render_camera, lights);

                program.use_uniform(
                    "view_projection",
                    render_camera
                        .projection
                        .projection_matrix(&render_camera.camera.viewport_or_default())
                        * render_camera.transform.view_matrix(),
                );

                let texture_width = (3 * max_instances as u32);
                let texture_height = (commands.len() as u32);

                //program.use_uniform("instance_texture_width", texture_width as f32);
                //program.use_uniform("instance_texture_height", texture_height as f32);

                program.use_texture("instance_texture", &instance_texture);

                gpu_mesh_manager.draw(program, render_states, render_camera.camera, commands);

            })
            .expect("Failed compiling shader");
    }

    fn vertex_shader_source(lights: &[&dyn Light]) -> String {
        let mut output = lights_shader_source(lights);
        output.push_str(include_str!("../shaders/mesh.vert"));

        output
    }
}
