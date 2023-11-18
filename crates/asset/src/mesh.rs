use std::fs;

use naia_serde::BitReader;

use math::Vec3;
use render_api::{AssetHash, base::{Indices, CpuMesh, Positions}};

#[derive(Hash)]
pub struct MeshFile {
    path: String,
}

impl AssetHash<CpuMesh> for MeshFile {}

impl MeshFile {
    pub fn load(path: &str) -> Self {
        Self {
            path: path.to_string(),
        }
    }
}

impl From<MeshFile> for CpuMesh {
    fn from(mesh_file: MeshFile) -> Self {

        let file_path = format!("assets/{}", &mesh_file.path);

        let data = fs::read(file_path).expect("unable to read file");

        let mut bit_reader = BitReader::new(&data);

        let mesh_actions = filetypes::MeshAction::read(&mut bit_reader).expect("unable to read mesh file");

        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for action in mesh_actions {
            match action {
                filetypes::MeshAction::Vertex(x, y, z) => {
                    println!("Vertex: {}, {}, {}", x, y, z);
                    let vertex = Vec3::new(x as f32, y as f32, z as f32);
                    vertices.push(vertex);
                }
                filetypes::MeshAction::Face(face_id, vertex_a_id, vertex_b_id, vertex_c_id, _, _, _) => {
                    println!("Face: {}, {}, {}, {}", face_id, vertex_a_id, vertex_b_id, vertex_c_id);
                    indices.push(vertex_a_id);
                    indices.push(vertex_b_id);
                    indices.push(vertex_c_id);
                }
                _ => {}
            }
        }

        let mut mesh = CpuMesh {
            indices: Indices(Some(indices)),
            positions: Positions(vertices),
            ..Default::default()
        };

        mesh.compute_normals();
        mesh
    }
}