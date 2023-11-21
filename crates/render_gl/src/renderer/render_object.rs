use std::collections::HashMap;
use gl::DrawArraysIndirectCommand;

use math::Mat4;
use render_api::{base::AxisAlignedBoundingBox, components::{CameraProjection, Transform}, Handle};
use render_api::base::{CpuMaterial, CpuMesh};
use render_api::components::Camera;

use crate::{AssetMapping, core::{Context, InstanceBuffer, Program, RenderStates}, GpuMesh, GpuMeshManager, renderer::{Instances, lights_shader_source, Light, Material, RenderCamera}};

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

    pub fn to_commands(self, gpu_mesh_manager: &GpuMeshManager) -> (Vec<DrawArraysIndirectCommand>, HashMap<String, InstanceBuffer>) {

        let mut instance_buffers: HashMap<String, InstanceBuffer> = Default::default();

        let mut commands = Vec::new();
        let mut base_instance = 0;
        for (mesh_handle, transforms) in self.mesh_handle_transform_map {
            let gpu_mesh = gpu_mesh_manager.get(&mesh_handle).unwrap();

            let count = gpu_mesh.count();
            let instance_count = transforms.len();
            let first = gpu_mesh.first();

            commands.push((mesh_handle, DrawArraysIndirectCommand::new(
                count as u32,
                instance_count as u32,
                first as u32,
                base_instance as u32,
            )));

            base_instance += instance_count;

            Self::instance_buffers_add(&mut instance_buffers, transforms);
        }

        commands.sort_by(|(a, _), (b, _)| a.cmp(b));

        // strip the handles from the commands
        let commands: Vec<DrawArraysIndirectCommand> = commands.into_iter().map(|(_, command)| command).collect();

        (commands, instance_buffers)
    }


    fn instance_buffers_add(instance_buffers: &mut HashMap<String, InstanceBuffer>, transforms: Vec<Mat4>) {
        let indices = {
            // No need to order, just return the indices as is.
            (0..transforms.len()).collect::<Vec<usize>>()
        };

        // Next, we can compute the instance buffers with that ordering.
        {
            let mut transform_row1 = Vec::new();
            let mut transform_row2 = Vec::new();
            let mut transform_row3 = Vec::new();
            for transformation in indices.iter().map(|i| transforms[*i]) {
                transform_row1.push(transformation.row(0));
                transform_row2.push(transformation.row(1));
                transform_row3.push(transformation.row(2));
            }

            instance_buffers.insert(
                "transform_row1".to_string(),
                InstanceBuffer::new_with_data(&transform_row1),
            );
            instance_buffers.insert(
                "transform_row2".to_string(),
                InstanceBuffer::new_with_data(&transform_row2),
            );
            instance_buffers.insert(
                "transform_row3".to_string(),
                InstanceBuffer::new_with_data(&transform_row3),
            );
        }
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
        let (commands, instance_buffers) = object.to_commands(gpu_mesh_manager);
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

                for attribute_name in ["transform_row1", "transform_row2", "transform_row3"] {
                    if program.requires_attribute(attribute_name) {
                        program.use_instance_attribute(
                            attribute_name,
                            instance_buffers
                                .get(attribute_name).unwrap_or_else(|| panic!("the render call requires the {} instance buffer which is missing on the given geometry", attribute_name)),
                        );
                    }
                }
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
