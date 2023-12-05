use std::fs;

use naia_serde::BitReader;

use math::Vec3;
use render_api::{base::CpuMesh, AssetHash, Handle, Assets};

use crate::asset_dependency::AssetDependency;

#[derive(Hash)]
struct MeshFilePath(String);

#[derive(Debug)]
pub struct MeshFile {
    path: AssetDependency<CpuMesh>,
}

impl Default for MeshFile {
    fn default() -> Self {
        panic!("");
    }
}

impl MeshFile {
    pub(crate) fn get_cpu_mesh_handle(&self) -> Option<&Handle<CpuMesh>> {
        if let AssetDependency::<CpuMesh>::Handle(handle) = &self.path {
            Some(handle)
        } else {
            None
        }
    }

    pub(crate) fn load_cpu_mesh(&mut self, meshes: &mut Assets<CpuMesh>) {
        let AssetDependency::<CpuMesh>::Path(path) = &self.path else {
            panic!("expected handle right after load");
        };
        let cpu_mesh_handle = meshes.add(MeshFilePath(path.clone()));
        self.path.load_handle(cpu_mesh_handle);
    }
}

impl AssetHash<CpuMesh> for MeshFilePath {}
impl AssetHash<MeshFile> for String {}

impl From<String> for MeshFile {
    fn from(path: String) -> Self {
        Self {
            path: AssetDependency::Path(path),
        }
    }
}

impl From<MeshFilePath> for CpuMesh {
    fn from(file_path: MeshFilePath) -> Self {
        let path = file_path.0;
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };
        //let data = include_bytes!("cube.mesh");

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::MeshAction::read(&mut bit_reader).expect("unable to parse file");

        let mut vertices = Vec::new();
        let mut positions = Vec::new();
        let mut face_indices = Vec::new();
        for action in actions {
            match action {
                filetypes::MeshAction::Vertex(x, y, z) => {
                    // println!("Vertex: {}, {}, {}", x, y, z);
                    let vertex = Vec3::new(x as f32, y as f32, z as f32);
                    vertices.push(vertex);
                }
                filetypes::MeshAction::Face(
                    face_id,
                    vertex_a_id,
                    vertex_b_id,
                    vertex_c_id,
                    _,
                    _,
                    _,
                ) => {
                    // println!(
                    //     "Face: {}, {}, {}, {}",
                    //     face_id, vertex_a_id, vertex_b_id, vertex_c_id
                    // );
                    let vertex_a = vertices[vertex_a_id as usize];
                    let vertex_b = vertices[vertex_b_id as usize];
                    let vertex_c = vertices[vertex_c_id as usize];

                    positions.push(vertex_a);
                    positions.push(vertex_b);
                    positions.push(vertex_c);

                    face_indices.push(face_id);
                    face_indices.push(face_id);
                    face_indices.push(face_id);
                }
                filetypes::MeshAction::Edge(_, _) => {
                    // do nothing
                }
            }
        }

        let mut mesh = CpuMesh::from_vertices(positions);
        mesh.add_face_indices(face_indices);
        mesh
    }
}
