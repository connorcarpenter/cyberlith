use std::{collections::HashMap, default::Default};

use bevy_ecs::system::Resource;

use gl::DrawArraysIndirectCommand;
use math::Vec3;
use render_api::{
    base::{AxisAlignedBoundingBox, CpuMesh},
    components::Camera,
};
use storage::Handle;

use crate::core::{Program, RenderStates, VertexBuffer};

#[derive(Resource)]
pub struct GpuMeshManager {
    assets: HashMap<Handle<CpuMesh>, GpuMesh>,
    gpu_positions: Option<VertexBuffer>,
    gpu_normals: Option<VertexBuffer>,
    gpu_face_indices: Option<VertexBuffer>,
    cpu_positions: Vec<Vec3>,
    cpu_normals: Vec<Vec3>,
    cpu_face_indices: Vec<u16>,
}

impl Default for GpuMeshManager {
    fn default() -> Self {
        Self {
            assets: HashMap::new(),
            gpu_positions: None,
            gpu_normals: None,
            gpu_face_indices: None,
            cpu_positions: Vec::new(),
            cpu_normals: Vec::new(),
            cpu_face_indices: Vec::new(),
        }
    }
}

impl GpuMeshManager {
    pub fn insert(&mut self, handle: Handle<CpuMesh>, cpu_mesh: &CpuMesh) {
        let new_cpu_vertices = cpu_mesh.vertices();
        let new_cpu_normals = cpu_mesh.compute_normals();
        let new_cpu_face_indices = cpu_mesh.face_indices();
        let new_aabb = cpu_mesh.compute_aabb();

        let first = self.cpu_positions.len();
        let count = new_cpu_vertices.len();

        let gpu_mesh = GpuMesh::new(first, count, new_aabb);
        self.assets.insert(handle, gpu_mesh);

        self.cpu_positions.extend(new_cpu_vertices);
        self.cpu_normals.extend(new_cpu_normals);
        self.cpu_face_indices.extend(new_cpu_face_indices);

        self.gpu_sync();
    }

    fn gpu_sync(&mut self) {
        if self.gpu_positions.is_none() {
            self.gpu_positions = Some(VertexBuffer::new());
            self.gpu_normals = Some(VertexBuffer::new());
            self.gpu_face_indices = Some(VertexBuffer::new());
        }
        let gpu_positions = self.gpu_positions.as_mut().unwrap();
        gpu_positions.fill(&self.cpu_positions);

        let gpu_normals = self.gpu_normals.as_mut().unwrap();
        gpu_normals.fill(&self.cpu_normals);

        let gpu_face_indices = self.gpu_face_indices.as_mut().unwrap();
        gpu_face_indices.fill(&self.cpu_face_indices);
    }

    pub fn get(&self, handle: &Handle<CpuMesh>) -> Option<&GpuMesh> {
        self.assets.get(&handle)
    }

    pub fn remove(&mut self, _handle: &Handle<CpuMesh>) -> Option<GpuMesh> {
        todo!();
        self.assets.remove(_handle)
    }

    pub fn is_ready(&self) -> bool {
        self.gpu_positions.is_some()
    }

    pub fn use_attributes(&self, program: &Program) {
        let gpu_positions = self.gpu_positions.as_ref().unwrap();
        let gpu_normals = self.gpu_normals.as_ref().unwrap();
        let gpu_face_indices = self.gpu_face_indices.as_ref().unwrap();

        program.use_vertex_attribute("vertex_world_position", gpu_positions);
        program.use_vertex_attribute_if_required("vertex_world_normal", gpu_normals);
        program.use_vertex_attribute("vertex_face_index", gpu_face_indices);
    }

    pub fn render(
        &self,
        program: &Program,
        render_states: RenderStates,
        camera: &Camera,
        draw_commands: Vec<DrawArraysIndirectCommand>,
    ) {
        self.use_attributes(program);

        program.multi_draw_arrays_indirect(
            render_states,
            camera.viewport_or_default(),
            draw_commands,
        );
    }
}

pub struct GpuMesh {
    first: usize,
    count: usize,
    aabb: AxisAlignedBoundingBox,
}

impl GpuMesh {
    pub fn new(first: usize, count: usize, aabb: AxisAlignedBoundingBox) -> Self {
        Self { first, count, aabb }
    }

    pub fn aabb(&self) -> AxisAlignedBoundingBox {
        self.aabb
    }

    pub fn first(&self) -> usize {
        self.first
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

//pub fn aabb(&self) -> AxisAlignedBoundingBox {
//         self.aabb
//     }
//
