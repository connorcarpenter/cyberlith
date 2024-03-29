use bevy_log::info;

use math::Vec3;
use render_api::base::CpuMesh;
use storage::{Handle, Storage};

use crate::asset_dependency::AssetDependency;

pub struct MeshData {
    path: AssetDependency<CpuMesh>,
}

impl Default for MeshData {
    fn default() -> Self {
        panic!("");
    }
}

impl MeshData {
    pub fn get_cpu_mesh_handle(&self) -> Option<&Handle<CpuMesh>> {
        if let AssetDependency::<CpuMesh>::Handle(handle) = &self.path {
            Some(handle)
        } else {
            None
        }
    }

    pub(crate) fn has_cpu_mesh(&self) -> bool {
        if let AssetDependency::<CpuMesh>::Handle(_) = &self.path {
            return true;
        }
        return false;
    }

    pub(crate) fn load_cpu_mesh(&mut self, meshes: &mut Storage<CpuMesh>) {
        let AssetDependency::<CpuMesh>::Bytes(bytes) = &self.path else {
            panic!("expected handle right after load");
        };

        let actions = asset_serde::bits::MeshAction::read(bytes).expect("unable to parse file");

        info!("--- reading mesh file ---",);

        let mut vertices = Vec::new();
        let mut positions = Vec::new();
        let mut face_indices = Vec::new();
        for action in actions {
            match action {
                asset_serde::bits::MeshAction::Vertex(x, y, z) => {
                    // info!("Vertex: {}, {}, {}", x, y, z);
                    let vertex = Vec3::new(x as f32, y as f32, z as f32);
                    vertices.push(vertex);
                }
                asset_serde::bits::MeshAction::Face(
                    face_id,
                    vertex_a_id,
                    vertex_b_id,
                    vertex_c_id,
                ) => {
                    // info!(
                    //     "Face: {}, {}, {}, {}",
                    //     face_id, vertex_a_id, vertex_b_id, vertex_c_id
                    // );
                    let vertex_a = vertices[vertex_a_id as usize];
                    let vertex_b = vertices[vertex_b_id as usize];
                    let vertex_c = vertices[vertex_c_id as usize];

                    positions.push(vertex_a);
                    positions.push(vertex_b);
                    positions.push(vertex_c);

                    info!("face_id: {}", face_id);

                    face_indices.push(face_id);
                    face_indices.push(face_id);
                    face_indices.push(face_id);
                }
            }
        }

        info!("--- done reading mesh file ---");

        let mut cpu_mesh = CpuMesh::from_vertices(positions);
        cpu_mesh.add_face_indices(face_indices);

        let cpu_mesh_handle = meshes.add_unique(cpu_mesh);

        self.path.load_handle(cpu_mesh_handle);
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let boxed_bytes = bytes.to_vec().into_boxed_slice();

        Self {
            path: AssetDependency::Bytes(boxed_bytes),
        }
    }
}
