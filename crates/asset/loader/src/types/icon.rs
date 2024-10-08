use std::collections::HashMap;

use math::Vec3;
use render_api::base::{CpuMaterial, CpuMesh, CpuSkin};
use storage::{Handle, Storage};

use crate::{asset_dependency::AssetDependency, AssetHandle, PaletteData, TypedAssetId};

pub struct IconData {
    palette_file: AssetDependency<PaletteData>,
    frames: Vec<Frame>,
    max_width: f32,
    max_height: f32,
}

impl Default for IconData {
    fn default() -> Self {
        panic!("");
    }
}

impl IconData {
    pub(crate) fn get_max_width(&self) -> f32 {
        self.max_width
    }

    pub(crate) fn get_max_height(&self) -> f32 {
        self.max_height
    }

    pub fn get_frame_width(&self, index: usize) -> Option<f32> {
        let frame = self.frames.get(index)?;
        Some(frame.get_width())
    }

    pub(crate) fn get_frame_height(&self, index: usize) -> Option<f32> {
        let frame = self.frames.get(index)?;
        Some(frame.get_height())
    }

    pub(crate) fn get_palette_file_handle(&self) -> Option<&AssetHandle<PaletteData>> {
        if let AssetDependency::<PaletteData>::AssetHandle(handle) = &self.palette_file {
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

    pub(crate) fn load_cpu_meshes(&mut self, meshes: &mut Storage<CpuMesh>) {
        // info!("icon: load_cpu_meshes");
        for frame in &mut self.frames {
            frame.load_cpu_mesh_handle(meshes);
        }
    }

    pub(crate) fn load_cpu_skins(
        &mut self,
        meshes: &Storage<CpuMesh>,
        materials: &Storage<CpuMaterial>,
        skins: &mut Storage<CpuSkin>,
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
        asset_handle: AssetHandle<Self>,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        let AssetDependency::<PaletteData>::AssetId(asset_id) = &self.palette_file else {
            panic!("expected asset_id right after load");
        };
        dependencies.push((asset_handle.into(), TypedAssetId::Palette(asset_id.clone())));
    }

    pub(crate) fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        match dependency_typed_id {
            TypedAssetId::Palette(asset_id) => {
                let asset_handle = AssetHandle::<PaletteData>::new(asset_id);
                self.palette_file.load_asset_handle(asset_handle);
                // info!("icon: load_palette");
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }

    pub(crate) fn has_all_dependencies(&self) -> bool {
        if let AssetDependency::<PaletteData>::AssetHandle(_) = &self.palette_file {
            return true;
        }
        return false;
    }

    pub fn get_cpu_mesh_handle(&self, subimage_index: usize) -> Option<&Handle<CpuMesh>> {
        let frame = &self.frames[subimage_index];
        frame.get_cpu_mesh_handle()
    }

    pub fn get_cpu_mesh_and_skin_handles(
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

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let actions = asset_serde::bits::IconAction::read(bytes).expect("unable to parse file");

        let mut palette_file_opt = None;
        let mut max_width = 0.0;
        let mut max_height = 0.0;
        let mut frames = Vec::new();
        for action in actions {
            match action {
                asset_serde::bits::IconAction::PaletteFile(asset_id) => {
                    palette_file_opt = Some(asset_id);
                }
                asset_serde::bits::IconAction::Frame(frame_actions) => {
                    // info!("- Frame Start: {} -", frames.len());

                    let mut vertices = Vec::new();
                    let mut positions = Vec::new();
                    let mut face_indices = Vec::new();
                    let mut face_color_ids = Vec::new();

                    for frame_action in frame_actions {
                        match frame_action {
                            asset_serde::bits::IconFrameAction::Vertex(x, y) => {
                                // info!("Vertex: ({}, {})", x, y);
                                let vertex = Vec3::new(x as f32, y as f32, 0.0);
                                vertices.push(vertex);
                            }
                            asset_serde::bits::IconFrameAction::Face(
                                face_id,
                                color_index,
                                vertex_a_id,
                                vertex_b_id,
                                vertex_c_id,
                            ) => {
                                let vertex_a = vertices[vertex_a_id as usize];
                                let vertex_b = vertices[vertex_b_id as usize];
                                let vertex_c = vertices[vertex_c_id as usize];

                                positions.push(vertex_a);
                                positions.push(vertex_b);
                                positions.push(vertex_c);

                                // info!("face_id: {}", face_id);

                                face_indices.push(face_id);
                                face_indices.push(face_id);
                                face_indices.push(face_id);

                                face_color_ids.push((face_id, color_index));
                            }
                        }
                    }

                    let mut mesh = CpuMesh::from_vertices(positions);
                    mesh.add_face_indices(face_indices);

                    let frame = Frame::new(mesh, face_color_ids);

                    let frame_width = frame.get_width();
                    let frame_height = frame.get_height();
                    if frame_width > max_width {
                        max_width = frame_width;
                    }
                    if frame_height > max_height {
                        max_height = frame_height;
                    }

                    frames.push(frame);

                    // info!("- Frame End -");
                }
            }
        }

        // todo: lots here

        Self {
            palette_file: AssetDependency::AssetId(palette_file_opt.unwrap()),
            frames,
            max_width,
            max_height,
        }
    }
}

struct FrameMetadata {
    // min_x: f32,
    // min_y: f32,
    // max_x: f32,
    // max_y: f32,
    width: f32,
    height: f32,
}

impl FrameMetadata {
    pub fn from_mesh(mesh: &CpuMesh) -> Self {
        let mut min_x = f32::MAX;
        let mut min_y = f32::MAX;
        let mut max_x = f32::MIN;
        let mut max_y = f32::MIN;

        for vertex in mesh.vertices() {
            if vertex.x < min_x {
                min_x = vertex.x;
            }
            if vertex.y < min_y {
                min_y = vertex.y;
            }
            if vertex.x > max_x {
                max_x = vertex.x;
            }
            if vertex.y > max_y {
                max_y = vertex.y;
            }
        }

        let width = max_x - min_x;
        let height = max_y - min_y;

        // info!("FrameMetadata: min_x: {}, min_y: {}, max_x: {}, max_y: {}, width: {}, height: {}", min_x, min_y, max_x, max_y, width, height);

        Self {
            // min_x,
            // min_y,
            // max_x,
            // max_y,
            width,
            height,
        }
    }
}

struct Frame {
    cpu_mesh: Option<CpuMesh>,
    cpu_mesh_handle: Option<Handle<CpuMesh>>,
    cpu_skin_handle: Option<Handle<CpuSkin>>,
    face_color_ids: Vec<(u16, u8)>,
    metadata: FrameMetadata,
}

impl Frame {
    fn new(cpu_mesh: CpuMesh, face_color_ids: Vec<(u16, u8)>) -> Self {
        let metadata = FrameMetadata::from_mesh(&cpu_mesh);
        Self {
            cpu_mesh: Some(cpu_mesh),
            cpu_mesh_handle: None,
            cpu_skin_handle: None,
            face_color_ids,
            metadata,
        }
    }

    pub fn get_width(&self) -> f32 {
        self.metadata.width
    }

    pub fn get_height(&self) -> f32 {
        self.metadata.height
    }

    fn get_cpu_mesh_handle(&self) -> Option<&Handle<CpuMesh>> {
        self.cpu_mesh_handle.as_ref()
    }

    fn has_cpu_mesh_handle(&self) -> bool {
        self.cpu_mesh_handle.is_some()
    }

    fn load_cpu_mesh_handle(&mut self, meshes: &mut Storage<CpuMesh>) {
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
        meshes: &Storage<CpuMesh>,
        materials: &Storage<CpuMaterial>,
        skins: &mut Storage<CpuSkin>,
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

            if face_id >= biggest_face_id {
                biggest_face_id = face_id + 1;
            }
        }

        let mut map = HashMap::new();
        for (face_id, color_id) in &self.face_color_ids {
            map.insert(*face_id, *color_id);
        }
        for index in 0..biggest_face_id {
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
