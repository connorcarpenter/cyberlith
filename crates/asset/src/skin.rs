use std::{collections::HashMap, fs};

use bevy_log::info;

use naia_serde::BitReader;

use render_api::{AssetHash, Assets, Handle, base::{CpuMaterial, CpuMesh, CpuSkin}};

use crate::{asset_handle::AssetHandleImpl, asset_dependency::AssetDependency, AssetHandle, MeshFile, PaletteData};

pub struct SkinData {
    mesh_file: AssetDependency<MeshFile>,
    palette_file: AssetDependency<PaletteData>,
    cpu_skin_handle: Option<Handle<CpuSkin>>,
    bckg_color_id: u8,
    // face_index, color_index
    face_color_ids: Vec<(u16, u8)>,
}

impl AssetHash<SkinData> for String {}

impl Default for SkinData {
    fn default() -> Self {
        panic!("");
    }
}

impl SkinData {
    pub(crate) fn get_mesh_file_handle(&self) -> Option<&Handle<MeshFile>> {
        if let AssetDependency::<MeshFile>::Handle(handle) = &self.mesh_file {
            Some(handle)
        } else {
            None
        }
    }

    pub(crate) fn get_palette_file_handle(&self) -> Option<&Handle<PaletteData>> {
        if let AssetDependency::<PaletteData>::Handle(handle) = &self.palette_file {
            Some(handle)
        } else {
            None
        }
    }

    pub(crate) fn get_cpu_skin_handle(&self) -> Option<&Handle<CpuSkin>> {
        self.cpu_skin_handle.as_ref()
    }

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

    pub(crate) fn finish_dependency(&mut self, _dependency_path: String, dependency_handle: AssetHandle) {
        match dependency_handle.to_impl() {
            AssetHandleImpl::Mesh(handle) => {
                self.mesh_file.load_handle(handle);
            }
            AssetHandleImpl::Palette(handle) => {
                self.palette_file.load_handle(handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }

    pub(crate) fn has_all_dependencies(&self) -> bool {
        if let AssetDependency::<MeshFile>::Handle(_) = &self.mesh_file {
            if let AssetDependency::<PaletteData>::Handle(_) = &self.palette_file {
                return true;
            }
        }
        return false;
    }

    pub(crate) fn load_cpu_skin(
        &mut self,
        materials: &Assets<CpuMaterial>,
        skins: &mut Assets<CpuSkin>,
        mesh_data: &CpuMesh,
        palette_data: &PaletteData
    ) -> bool {
        let mut new_skin = CpuSkin::default();

        if !mesh_data.is_skinnable() {
            panic!("invalid mesh for skin");
        }

        let mut biggest_face_id = 0;
        let mut map = HashMap::new();
        let mesh_face_ids = mesh_data.face_indices();
        for index in 0..mesh_face_ids.len() / 3 {
            let face_id = mesh_face_ids[index * 3];
            map.insert(face_id, self.bckg_color_id);

            if face_id > biggest_face_id {
                biggest_face_id = face_id;
            }
        }
        for (face_id, color_id) in &self.face_color_ids {
            let Some(value) = map.get_mut(face_id) else {
                panic!("invalid face id, {}", face_id);
            };
            *value = *color_id;
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

impl From<String> for SkinData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::SkinAction::read(&mut bit_reader).expect("unable to parse file");

        let mut face_color_ids = Vec::new();
        let mut bck_color_index = None;
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
                    face_color_ids.push((face_index, color_index));
                }
            }
        }


        info!("--- reading skin: {} ---", path);

        // for (face_id, color_id) in color_ids.iter().enumerate() {
        //     info!("face {} -> color {}", face_id, color_id);
        // }

        info!("--- done reading skin ---");

        Self {
            mesh_file: AssetDependency::Path(mesh_file_opt.unwrap()),
            palette_file: AssetDependency::Path(palette_file_opt.unwrap()),
            cpu_skin_handle: None,
            bckg_color_id: bck_color_index.unwrap(),
            face_color_ids,
        }
    }
}