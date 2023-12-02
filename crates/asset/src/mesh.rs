use std::fs;

use naia_serde::BitReader;

use math::Vec3;
use render_api::{base::CpuMesh, AssetHash};

pub struct MeshFile {
    path: String,
}

impl Default for MeshFile {
    fn default() -> Self {
        panic!("");
    }
}

impl MeshFile {

}

impl AssetHash<MeshFile> for String {}

impl From<String> for MeshFile {
    fn from(path: String) -> Self {
        Self {
            path
        }
    }
}

impl From<MeshFile> for CpuMesh {
    fn from(file: MeshFile) -> Self {
        let file_path = format!("assets/{}", &file.path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };
        //let data = include_bytes!("cube.mesh");

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::MeshAction::read(&mut bit_reader).expect("unable to parse file");

        let mut vertices = Vec::new();
        let mut positions = Vec::new();
        for action in actions {
            match action {
                filetypes::MeshAction::Vertex(x, y, z) => {
                    // println!("Vertex: {}, {}, {}", x, y, z);
                    let vertex = Vec3::new(x as f32, y as f32, z as f32);
                    vertices.push(vertex);
                }
                filetypes::MeshAction::Face(
                    _face_id,
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

                    // TODO: probably need to pass in the face_id here, for vertex attributes?
                    positions.push(vertex_a);
                    positions.push(vertex_b);
                    positions.push(vertex_c);
                }
                filetypes::MeshAction::Edge(_, _) => {
                    // do nothing
                }
            }
        }

        CpuMesh::from_vertices(positions)
    }
}
