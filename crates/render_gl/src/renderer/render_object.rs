use std::collections::HashMap;
use gl::DrawArraysIndirectCommand;

use math::{Mat4, Vec4};
use render_api::{base::AxisAlignedBoundingBox, components::{CameraProjection, Transform}, Handle};
use render_api::base::{CpuMaterial, CpuMesh};
use render_api::components::Camera;

use crate::{AssetMapping, core::{Context, InstanceBuffer, Program, RenderStates}, GpuMesh, GpuMeshManager, renderer::{Instances, lights_shader_source, Light, Material, RenderCamera}};
use crate::core::GpuTexture2D;

// Render Object
#[derive(Clone)]
pub struct RenderObject {
    mesh_handle_transform_map: HashMap<Handle<CpuMesh>, Vec<Mat4>>,
}

impl RenderObject{
    pub fn new() -> Self {
        Self {
            mesh_handle_transform_map: HashMap::new(),
        }
    }

    pub fn add_transform(&mut self, mesh_handle: &Handle<CpuMesh>, transform: &Transform) {
        if !self.mesh_handle_transform_map.contains_key(mesh_handle) {
            self.mesh_handle_transform_map.insert(*mesh_handle, Vec::new());
        }
        let map = self.mesh_handle_transform_map.get_mut(mesh_handle).unwrap();
        map.push(transform.compute_matrix());
    }

    pub fn to_commands(self, gpu_mesh_manager: &GpuMeshManager) -> (Vec<DrawArraysIndirectCommand>, Vec<[f32; 4]>, usize) {

        let mut mesh_handle_transform_map = self.mesh_handle_transform_map;

        let mut commands = Vec::new();
        let mut transform_rows = Vec::new();
        let mut max_instance_count = 0;

        let mut mesh_handles = mesh_handle_transform_map.keys().map(|handle| *handle).collect::<Vec<_>>();
        mesh_handles.sort();

        for mesh_handle in mesh_handles {
            let transforms = mesh_handle_transform_map.remove(&mesh_handle).unwrap();
            let gpu_mesh = gpu_mesh_manager.get(&mesh_handle).unwrap();

            let count = gpu_mesh.count();
            let instance_count = transforms.len();
            max_instance_count = max_instance_count.max(instance_count);
            let first = gpu_mesh.first();

            commands.push(DrawArraysIndirectCommand::new(
                count as u32,
                instance_count as u32,
                first as u32,
                0,
            ));

            let transform_row = Self::get_transform_row(transforms);
            transform_rows.push(transform_row);
        }

        // convert transform rows to grid
        let mut transform_grid: Vec<[f32; 4]> = Vec::new();
        let transform_grid_width = max_instance_count * 3;
        for transform_row in transform_rows {
            let row_count = transform_row.len();
            let pad_count = transform_grid_width - row_count;
            for transform_cell in transform_row {
                transform_grid.push(transform_cell);
            }
            for _ in 0..pad_count {
                transform_grid.push([0.0; 4]);
            }
        }

        (commands, transform_grid, max_instance_count)
    }


    fn get_transform_row(transforms: Vec<Mat4>) -> Vec<[f32; 4]> {

        let mut transform_row = Vec::new();

        let indices = {
            // No need to order, just return the indices as is.
            (0..transforms.len()).collect::<Vec<usize>>()
        };

        // Next, we can compute the instance buffers with that ordering.
        {
            for transformation in indices.iter().map(|i| transforms[*i]) {
                transform_row.push(transformation.row(0).to_array());
                transform_row.push(transformation.row(1).to_array());
                transform_row.push(transformation.row(2).to_array());
            }
        }

        transform_row
    }
}

// Instanced rendering
pub struct RenderObjectInstanced;

impl RenderObjectInstanced {
    pub fn render<'a>(
        gpu_mesh_manager: &'a GpuMeshManager,
        materials: &'a AssetMapping<CpuMaterial, Box<dyn Material>>,
        render_camera: &'a RenderCamera<'a>,
        lights: &[&dyn Light],
        mat_handle: Handle<CpuMaterial>,
        object: RenderObject,
    ) {
        let (commands, transform_grid, max_instances) = object.to_commands(gpu_mesh_manager);
        let material = materials.get(&mat_handle).unwrap();

        let render_states = material.render_states();
        let fragment_shader = material.fragment_shader();
        let vertex_shader_source = Self::vertex_shader_source(lights);
        Context::get()
            .program(vertex_shader_source, fragment_shader.source, |program| {
                material.use_uniforms(program, render_camera, lights);

                program.use_uniform(
                    "view_projection",
                    render_camera
                        .projection
                        .projection_matrix(&render_camera.camera.viewport_or_default())
                        * render_camera.transform.view_matrix(),
                );

                let texture_width = (3 * max_instances as u32);
                let texture_height = (commands.len() as u32);

                program.use_uniform("instance_texture_width", texture_width as f32);
                program.use_uniform("instance_texture_height", texture_height as f32);

                let mut instances_texture = GpuTexture2D::new_empty::<[f32; 4]>(texture_width, texture_height);
                instances_texture.fill_pure(&transform_grid);

                program.use_texture("instance_texture", &instances_texture);

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
