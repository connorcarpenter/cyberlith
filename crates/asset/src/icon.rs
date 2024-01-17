use std::{collections::HashMap, fs};

use bevy_log::info;

use math::Vec3;
use naia_serde::BitReader;

use render_api::{
    base::{CpuMaterial, CpuMesh, CpuSkin},
    AssetHash, Assets, Handle,
};

use crate::{
    asset_dependency::AssetDependency, asset_handle::AssetHandleImpl, AssetHandle, PaletteData,
};

impl AssetHash<IconData> for String {}

pub struct IconData {
    palette_file: AssetDependency<PaletteData>,
    frames: Vec<Frame>,
}

impl Default for IconData {
    fn default() -> Self {
        panic!("");
    }
}

impl IconData {
    pub(crate) fn get_palette_file_handle(&self) -> Option<&Handle<PaletteData>> {
        if let AssetDependency::<PaletteData>::Handle(handle) = &self.palette_file {
            Some(handle)
        } else {
            None
        }
    }

    pub(crate) fn has_all_cpu_meshes(&self) -> bool {
        for frame in &self.frames {
            if !frame.has_cpu_mesh_handle() {
                return false;
            }
        }
        return true;
    }

    pub(crate) fn load_cpu_meshes(&mut self, meshes: &mut Assets<CpuMesh>) {
        info!("icon: load_cpu_meshes");
        for frame in &mut self.frames {
            frame.load_cpu_mesh_handle(meshes);
        }
    }

    pub(crate) fn load_cpu_skins(
        &mut self,
        meshes: &Assets<CpuMesh>,
        materials: &Assets<CpuMaterial>,
        skins: &mut Assets<CpuSkin>,
        palette_data: &PaletteData,
    ) -> bool {
        for frame in &mut self.frames {
            if !frame.has_cpu_skin_handle() {
                if !frame.load_cpu_skin(meshes, materials, skins, palette_data) {
                    return false;
                }
            }
        }
        return true;
    }

    pub(crate) fn load_dependencies(
        &self,
        handle: Handle<Self>,
        dependencies: &mut Vec<(AssetHandle, String)>,
    ) {
        let AssetDependency::<PaletteData>::Path(path) = &self.palette_file else {
            panic!("expected path right after load");
        };
        dependencies.push((handle.into(), path.clone()));
    }

    pub(crate) fn finish_dependency(
        &mut self,
        _dependency_path: String,
        dependency_handle: AssetHandle,
    ) {
        match dependency_handle.to_impl() {
            AssetHandleImpl::Palette(handle) => {
                self.palette_file.load_handle(handle);
                info!("icon: load_palette");
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }

    pub(crate) fn has_all_dependencies(&self) -> bool {
        if let AssetDependency::<PaletteData>::Handle(_) = &self.palette_file {
            return true;
        }
        return false;
    }

    pub(crate) fn get_cpu_mesh_and_skin_handles(
        &self,
        subimage_index: usize,
    ) -> Option<(Handle<CpuMesh>, Handle<CpuSkin>)> {
        let frame = &self.frames[subimage_index];
        if frame.has_cpu_mesh_handle() && frame.has_cpu_skin_handle() {
            return Some((
                frame.get_cpu_mesh_handle().unwrap().clone(),
                frame.get_cpu_skin_handle().unwrap().clone(),
            ));
        }
        return None;
    }

    pub(crate) fn get_subimage_count(&self) -> usize {
        self.frames.len()
    }
}

struct Frame {
    cpu_mesh: Option<CpuMesh>,
    cpu_mesh_handle: Option<Handle<CpuMesh>>,
    cpu_skin_handle: Option<Handle<CpuSkin>>,
    face_color_ids: Vec<(u16, u8)>,
}

impl Frame {
    fn new(cpu_mesh: CpuMesh, face_color_ids: Vec<(u16, u8)>) -> Self {
        Self {
            cpu_mesh: Some(cpu_mesh),
            cpu_mesh_handle: None,
            cpu_skin_handle: None,
            face_color_ids,
        }
    }

    fn get_cpu_mesh_handle(&self) -> Option<&Handle<CpuMesh>> {
        self.cpu_mesh_handle.as_ref()
    }

    fn has_cpu_mesh_handle(&self) -> bool {
        self.cpu_mesh_handle.is_some()
    }

    fn load_cpu_mesh_handle(&mut self, meshes: &mut Assets<CpuMesh>) {
        let cpu_mesh = self.cpu_mesh.take().unwrap();
        let cpu_mesh_handle = meshes.add_unique(cpu_mesh);
        self.cpu_mesh_handle = Some(cpu_mesh_handle);
    }

    fn has_cpu_skin_handle(&self) -> bool {
        self.cpu_skin_handle.is_some()
    }

    fn get_cpu_skin_handle(&self) -> Option<&Handle<CpuSkin>> {
        self.cpu_skin_handle.as_ref()
    }

    pub(crate) fn load_cpu_skin(
        &mut self,
        meshes: &Assets<CpuMesh>,
        materials: &Assets<CpuMaterial>,
        skins: &mut Assets<CpuSkin>,
        palette_data: &PaletteData,
    ) -> bool {
        let mut new_skin = CpuSkin::default();

        let mut biggest_face_id = 0;
        let mesh_face_ids = if let Some(cpu_mesh) = self.cpu_mesh.as_ref() {
            cpu_mesh.face_indices()
        } else {
            if let Some(cpu_mesh_handle) = self.cpu_mesh_handle.as_ref() {
                let cpu_mesh = meshes.get(cpu_mesh_handle).unwrap();
                cpu_mesh.face_indices()
            } else {
                panic!("no cpu mesh or handle");
            }
        };
        for index in 0..mesh_face_ids.len() / 3 {
            let face_id = mesh_face_ids[index * 3];

            if face_id > biggest_face_id {
                biggest_face_id = face_id;
            }
        }

        let mut map = HashMap::new();
        for (face_id, color_id) in &self.face_color_ids {
            map.insert(*face_id, *color_id);
        }
        for index in 0..=biggest_face_id {
            let Some(color_id) = map.get(&index) else {
                panic!("invalid face id, {}", index);
            };

            let cpu_material_handle = palette_data.get_cpu_mat_handle(*color_id as usize);

            if !materials.added_was_flushed(&cpu_material_handle) {
                // still waiting on the CpuMaterial to be loaded into a GpuMaterial ... need to wait
                return false;
            }

            new_skin.add_face_color(cpu_material_handle);
        }

        new_skin.log();

        let new_handle = skins.add_unique(new_skin);
        self.cpu_skin_handle = Some(new_handle);

        // success!
        return true;
    }
}

impl From<String> for IconData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let mut bit_reader = BitReader::new(&data);

        let actions = filetypes::IconAction::read(&mut bit_reader).expect("unable to parse file");

        let mut palette_file_opt = None;
        let mut frames = Vec::new();
        for action in actions {
            match action {
                filetypes::IconAction::PaletteFile(path, file_name) => {
                    palette_file_opt = Some(format!("{}/{}", path, file_name));
                }
                filetypes::IconAction::Frame(frame_actions) => {
                    info!("- Frame Start: {} -", frames.len());

                    let mut vertices = Vec::new();
                    let mut positions = Vec::new();
                    let mut face_indices = Vec::new();
                    let mut face_color_ids = Vec::new();

                    for frame_action in frame_actions {
                        match frame_action {
                            filetypes::IconFrameAction::Vertex(x, y) => {
                                info!("Vertex: ({}, {})", x, y);
                                let vertex = Vec3::new(x as f32, y as f32, 0.0);
                                vertices.push(vertex);
                            }
                            filetypes::IconFrameAction::Face(
                                face_id,
                                color_index,
                                vertex_a_id,
                                vertex_b_id,
                                vertex_c_id,
                                _,
                                _,
                                _,
                            ) => {
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

                                face_color_ids.push((face_id, color_index));
                            }
                            filetypes::IconFrameAction::Edge(_, _) => {
                                // do nothing
                            }
                        }
                    }

                    let mut mesh = CpuMesh::from_vertices(positions);
                    mesh.add_face_indices(face_indices);

                    let frame = Frame::new(mesh, face_color_ids);
                    frames.push(frame);

                    info!("- Frame End -");
                }
            }
        }

        // todo: lots here

        Self {
            palette_file: AssetDependency::Path(palette_file_opt.unwrap()),
            frames,
        }
    }
}
