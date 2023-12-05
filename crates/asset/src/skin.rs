use std::collections::HashMap;
use std::fs;
use bevy_log::info;

use naia_serde::BitReader;

use render_api::{AssetHash, Assets, Handle};
use render_api::base::{CpuMaterial, CpuSkin};

use crate::{asset_handle::AssetHandleImpl, asset_dependency::AssetDependency, AssetHandle, MeshFile, PaletteData};

impl AssetHash<SkinData> for String {}

impl Default for SkinData {
    fn default() -> Self {
        panic!("");
    }
}

#[derive(Debug)]
pub struct SkinData {
    mesh_file: AssetDependency<MeshFile>,
    palette_file: AssetDependency<PaletteData>,
    cpu_skin_handle: Option<Handle<CpuSkin>>,
    face_color_ids: Vec<u8>,
}

impl SkinData {
    pub(crate) fn get_mesh_file_handle(&self) -> Option<&Handle<MeshFile>> {
        if let AssetDependency::<MeshFile>::Handle(handle) = &self.mesh_file {
            Some(handle)
        } else {
            None
        }
    }

    pub(crate) fn get_cpu_skin_handle(&self) -> Option<&Handle<CpuSkin>> {
        self.cpu_skin_handle.as_ref()
    }
}

impl SkinData {

    pub(crate) fn load_dependencies(&self, handle: Handle<Self>, dependencies: &mut Vec<(AssetHandle, String)>) {
        let AssetDependency::<MeshFile>::Path(path) = &self.mesh_file else {
            panic!("expected path right after load");
        };
        dependencies.push((handle.into(), path.clone()));

        let AssetDependency::<PaletteData>::Path(path) = &self.palette_file else {
            panic!("expected path right after load");
        };
        dependencies.push((handle.into(), path.clone()));
    }

    pub(crate) fn finish_dependency(&mut self, _dependency_path: String, dependency_handle: AssetHandle) -> Option<Handle<PaletteData>> {
        match dependency_handle.to_impl() {
            AssetHandleImpl::Mesh(handle) => {
                self.mesh_file.load_handle(handle);
                return None;
            }
            AssetHandleImpl::Palette(handle) => {
                self.palette_file.load_handle(handle.clone());
                return Some(handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }

    pub(crate) fn load_cpu_skin(&mut self, materials: &Assets<CpuMaterial>, skins: &mut Assets<CpuSkin>, palette_data: &PaletteData) -> bool {
        let mut new_skin = CpuSkin::default();

        for face_color_id in &self.face_color_ids {
            let face_color_id = *face_color_id as usize;
            let cpu_material_handle = palette_data.get_cpu_mat_handle(face_color_id);

            if !materials.added_was_flushed(&cpu_material_handle) {
                // still waiting on the CpuMaterial to be loaded into a GpuMaterial ... need to wait
                return false;
            }

            new_skin.add_face_color(cpu_material_handle);
        }

        let new_handle = skins.add_unique(new_skin);
        self.cpu_skin_handle = Some(new_handle);

        // success!
        return true;
    }
}

impl From<String> for SkinData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::SkinAction::read(&mut bit_reader).expect("unable to parse file");

        let mut skin_colors = HashMap::new();
        let mut bck_color_index = None;
        let mut max_face_id = 0;
        let mut palette_file_opt = None;
        let mut mesh_file_opt = None;
        for action in actions {
            match action {
                filetypes::SkinAction::PaletteFile(path, file_name) => {
                    palette_file_opt = Some(format!("{}/{}", path, file_name));
                }
                filetypes::SkinAction::MeshFile(path, file_name) => {
                    mesh_file_opt = Some(format!("{}/{}", path, file_name));
                }
                filetypes::SkinAction::BackgroundColor(color_index) => {
                    bck_color_index = Some(color_index);
                }
                filetypes::SkinAction::SkinColor(face_index, color_index) => {
                    if face_index > max_face_id {
                        max_face_id = face_index;
                    }
                    skin_colors.insert(face_index, color_index);
                }
            }
        }

        let mut color_ids = Vec::new();
        for face_index in 0..=max_face_id {
            if let Some(color_index) = skin_colors.get(&face_index) {
                color_ids.push(*color_index);
            } else {
                color_ids.push(bck_color_index.unwrap());
            }
        }

        info!("--- reading skin: {} ---", path);

        for (face_id, color_id) in color_ids.iter().enumerate() {
            info!("face {} -> color {}", face_id, color_id);
        }

        info!("--- done reading skin ---");

        Self {
            mesh_file: AssetDependency::Path(mesh_file_opt.unwrap()),
            palette_file: AssetDependency::Path(palette_file_opt.unwrap()),
            cpu_skin_handle: None,
            face_color_ids: color_ids,
        }
    }
}