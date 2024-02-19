use bevy_log::info;

use asset_id::AssetId;
use math::Vec3;
use render_api::base::CpuMesh;
use storage::{AssetHash, Storage, Handle};

use crate::asset_dependency::AssetDependency;
use crate::data_from_asset_id;

#[derive(Hash)]
struct MeshAssetId(AssetId);

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

    pub(crate) fn has_cpu_mesh(&self) -> bool {
        if let AssetDependency::<CpuMesh>::Handle(_) = &self.path {
            return true;
        }
        return false;
    }

    pub(crate) fn load_cpu_mesh(&mut self, meshes: &mut Storage<CpuMesh>) {
        let AssetDependency::<CpuMesh>::AssetId(asset_id) = &self.path else {
            panic!("expected handle right after load");
        };
        let cpu_mesh_handle = meshes.add(MeshAssetId(asset_id.clone()));
        self.path.load_handle(cpu_mesh_handle);
    }
}

impl AssetHash<CpuMesh> for MeshAssetId {}
impl AssetHash<MeshFile> for AssetId {}

impl From<AssetId> for MeshFile {
    fn from(asset_id: AssetId) -> Self {
        Self {
            path: AssetDependency::AssetId(asset_id),
        }
    }
}

impl From<MeshAssetId> for CpuMesh {
    fn from(asset_id: MeshAssetId) -> Self {
        let asset_id = asset_id.0;

        let Ok(data) = data_from_asset_id(&asset_id) else {
            panic!("unable to read asset_id: {:?}", asset_id);
        };
        //let data = include_bytes!("cube.mesh");

        let actions = asset_io::bits::MeshAction::read(&data).expect("unable to parse file");

        info!("--- reading mesh file: {:?} ---", asset_id);

        let mut vertices = Vec::new();
        let mut positions = Vec::new();
        let mut face_indices = Vec::new();
        for action in actions {
            match action {
                asset_io::bits::MeshAction::Vertex(x, y, z) => {
                    // info!("Vertex: {}, {}, {}", x, y, z);
                    let vertex = Vec3::new(x as f32, y as f32, z as f32);
                    vertices.push(vertex);
                }
                asset_io::bits::MeshAction::Face(
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

        let mut mesh = CpuMesh::from_vertices(positions);
        mesh.add_face_indices(face_indices);
        mesh
    }
}
