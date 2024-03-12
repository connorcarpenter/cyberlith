use std::collections::HashMap;

use rand;
use rand::seq::SliceRandom;

use gl::DrawArraysIndirectCommand;
use math::Mat4;
use render_api::{
    base::CpuMesh,
    components::{Camera, CameraProjection, Projection, Transform, Viewport},
    resources::{MaterialOrSkinHandle, RenderPass},
};
use storage::Handle;

use crate::{
    core::{Context, Cull, GpuTexture2D, RenderStates},
    renderer::{lights_shader_source, Light},
    GpuMaterialManager, GpuMeshManager, GpuSkinManager,
};

pub trait RenderTargetExt {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn write(&self, render: impl FnOnce()) -> &Self;

    fn render(
        &self,
        gpu_mesh_manager: &GpuMeshManager,
        gpu_material_manager: &GpuMaterialManager,
        gpu_skin_manager: &GpuSkinManager,
        render_pass: RenderPass,
    ) -> &Self {
        let RenderPass {
            camera_opt,
            camera_transform_opt,
            camera_projection_opt,
            lights,
            meshes,
        } = render_pass;
        let camera = camera_opt.unwrap();
        let camera_transform = camera_transform_opt.unwrap();
        let camera_projection = camera_projection_opt.unwrap();

        let light_refs: Vec<&dyn Light> = lights.iter().map(|item| item as &dyn Light).collect();

        self.write(|| {
            render(
                gpu_mesh_manager,
                gpu_material_manager,
                gpu_skin_manager,
                &camera,
                &camera_transform,
                &camera_projection,
                &light_refs,
                meshes,
            );
        });
        self
    }

    ///
    /// Returns the viewport that encloses the entire target.
    ///
    fn viewport(&self) -> Viewport {
        Viewport::new_at_origin(self.width(), self.height())
    }
}

fn render<'a>(
    gpu_mesh_manager: &'a GpuMeshManager,
    gpu_material_manager: &'a GpuMaterialManager,
    gpu_skin_manager: &'a GpuSkinManager,
    camera: &Camera,
    camera_transform: &Transform,
    camera_projection: &Projection,
    lights: &[&dyn Light],
    meshes: HashMap<Handle<CpuMesh>, Vec<(MaterialOrSkinHandle, Mat4)>>,
) {
    if !gpu_skin_manager.is_ready()
        || !gpu_material_manager.is_ready()
        || !gpu_mesh_manager.is_ready()
    {
        return;
    }
    let (commands, instance_texture) = meshes_to_commands(
        meshes,
        gpu_mesh_manager,
        gpu_material_manager,
        gpu_skin_manager,
    );

    let render_states = RenderStates {
        cull: Cull::Back,
        ..Default::default()
    };
    let fragment_shader = gpu_material_manager.fragment_shader();
    let vertex_shader_source = vertex_shader_source(lights);
    Context::get()
        .program(vertex_shader_source, fragment_shader.source, |program| {
            gpu_material_manager.use_uniforms(
                program,
                camera,
                camera_transform,
                camera_projection,
                lights,
            );
            gpu_skin_manager.use_uniforms(program);

            program.use_uniform(
                "view_projection",
                camera_projection.projection_matrix(&camera.viewport_or_default())
                    * camera_transform.view_matrix(),
            );

            program.use_texture("instance_texture", &instance_texture);

            gpu_mesh_manager.render(program, render_states, camera, commands);
        })
        .expect("Failed compiling shader");
}

fn meshes_to_commands(
    mut mesh_handle_transform_map: HashMap<Handle<CpuMesh>, Vec<(MaterialOrSkinHandle, Mat4)>>,
    gpu_mesh_manager: &GpuMeshManager,
    gpu_mat_manager: &GpuMaterialManager,
    gpu_skin_manager: &GpuSkinManager,
) -> (Vec<DrawArraysIndirectCommand>, GpuTexture2D) {
    let mut commands = Vec::new();
    let mut instances_rows = Vec::new();
    let mut max_instance_count = 0;

    let mut mesh_handles = mesh_handle_transform_map
        .keys()
        .map(|handle| *handle)
        .collect::<Vec<_>>();
    mesh_handles.sort(); // TODO: is this still necessary??

    for mesh_handle in mesh_handles {
        let mut instances = mesh_handle_transform_map.remove(&mesh_handle).unwrap();

        if instances.len() > 4096 {
            let mut rng = rand::thread_rng();
            instances.shuffle(&mut rng);
            instances.truncate(4096);
        }

        //info!("Mesh has {} instances", instances.len());

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

        let instance_row = get_instance_row(gpu_mat_manager, gpu_skin_manager, instances);
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
    let mut instances_texture = GpuTexture2D::new_empty::<[f32; 4]>(texture_width, texture_height);
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
