use std::collections::HashMap;

// use logging::info;

use render_api::base::{CpuMaterial, CpuMesh, CpuSkin};
use storage::{Handle, Storage};

use crate::{asset_dependency::AssetDependency, AssetHandle, MeshData, PaletteData, TypedAssetId};

pub struct SkinData {
    mesh_file: AssetDependency<MeshData>,
    palette_file: AssetDependency<PaletteData>,
    cpu_skin_handle: Option<Handle<CpuSkin>>,
    bckg_color_id: u8,
    // face_index, color_index
    face_color_ids: Vec<(u16, u8)>,
}

impl Default for SkinData {
    fn default() -> Self {
        panic!("");
    }
}

impl SkinData {
    pub fn get_mesh_file_handle(&self) -> Option<&AssetHandle<MeshData>> {
        if let AssetDependency::<MeshData>::AssetHandle(handle) = &self.mesh_file {
            Some(handle)
        } else {
            None
        }
    }

    pub(crate) fn get_palette_file_handle(&self) -> Option<&AssetHandle<PaletteData>> {
        if let AssetDependency::<PaletteData>::AssetHandle(handle) = &self.palette_file {
            Some(handle)
        } else {
            None
        }
    }

    pub fn get_cpu_skin_handle(&self) -> Option<&Handle<CpuSkin>> {
        self.cpu_skin_handle.as_ref()
    }

    pub(crate) fn load_dependencies(
        &self,
        handle: AssetHandle<Self>,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        let AssetDependency::<MeshData>::AssetId(asset_id) = &self.mesh_file else {
            panic!("expected path right after load");
        };
        dependencies.push((handle.into(), TypedAssetId::Mesh(asset_id.clone())));

        let AssetDependency::<PaletteData>::AssetId(asset_id) = &self.palette_file else {
            panic!("expected path right after load");
        };
        dependencies.push((handle.into(), TypedAssetId::Palette(asset_id.clone())));
    }

    pub(crate) fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        match dependency_typed_id {
            TypedAssetId::Mesh(asset_id) => {
                let handle = AssetHandle::<MeshData>::new(asset_id);
                self.mesh_file.load_asset_handle(handle);
            }
            TypedAssetId::Palette(asset_id) => {
                let handle = AssetHandle::<PaletteData>::new(asset_id);
                self.palette_file.load_asset_handle(handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }

    pub(crate) fn has_all_dependencies(&self) -> bool {
        if let AssetDependency::<MeshData>::AssetHandle(_) = &self.mesh_file {
            if let AssetDependency::<PaletteData>::AssetHandle(_) = &self.palette_file {
                return true;
            }
        }
        return false;
    }

    pub(crate) fn load_cpu_skin(
        &mut self,
        materials: &Storage<CpuMaterial>,
        skins: &mut Storage<CpuSkin>,
        mesh_data: &CpuMesh,
        palette_data: &PaletteData,
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

    pub fn from_bytes(bytes: &[u8]) -> Self {
        // info!("--- reading skin ---");

        let actions = asset_serde::bits::SkinAction::read(bytes).expect("unable to parse file");

        let mut face_color_ids = Vec::new();
        let mut bck_color_index = None;
        let mut palette_file_opt = None;
        let mut mesh_file_opt = None;
        for action in actions {
            match action {
                asset_serde::bits::SkinAction::PaletteFile(asset_id) => {
                    // info!("palette file: {:?}", asset_id);
                    palette_file_opt = Some(asset_id);
                }
                asset_serde::bits::SkinAction::MeshData(asset_id) => {
                    // info!("mesh file: {:?}", asset_id);
                    mesh_file_opt = Some(asset_id);
                }
                asset_serde::bits::SkinAction::BackgroundColor(color_index) => {
                    // info!("background color: {}", color_index);
                    bck_color_index = Some(color_index);
                }
                asset_serde::bits::SkinAction::SkinColor(face_index, color_index) => {
                    // info!("face color: {} -> {}", face_index, color_index);
                    face_color_ids.push((face_index, color_index));
                }
            }
        }

        // for (face_id, color_id) in color_ids.iter().enumerate() {
        //     info!("face {} -> color {}", face_id, color_id);
        // }

        // info!("--- done reading skin ---");

        Self {
            mesh_file: AssetDependency::AssetId(mesh_file_opt.unwrap()),
            palette_file: AssetDependency::AssetId(palette_file_opt.unwrap()),
            cpu_skin_handle: None,
            bckg_color_id: bck_color_index.unwrap(),
            face_color_ids,
        }
    }
}
