use std::fs;

use naia_serde::BitReader;

use render_api::{AssetHash, Handle};

use crate::{asset_dependency::AssetDependency, AssetHandle, MeshFile, PaletteData};

impl AssetHash<SkinData> for String {}

impl Default for SkinData {
    fn default() -> Self {
        panic!("");
    }
}

pub struct SkinData {
    mesh_file: AssetDependency<MeshFile>,
    palette_file: AssetDependency<PaletteData>,
}

impl SkinData {

    pub fn load_dependencies(&self, handle: Handle<Self>, dependencies: &mut Vec<(AssetHandle, String)>) {
        let AssetDependency::<MeshFile>::Path(path) = &self.mesh_file else {
            panic!("expected path right after load");
        };
        dependencies.push((handle.into(), path.clone()));

        let AssetDependency::<PaletteData>::Path(path) = &self.palette_file else {
            panic!("expected path right after load");
        };
        dependencies.push((handle.into(), path.clone()));
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

        let mut palette_file_opt = None;
        let mut mesh_file_opt = None;
        for action in actions {
            match action {
                filetypes::SkinAction::PaletteFile(path, file_name) => {
                    println!("PaletteFile: {}/{}", path, file_name);
                    palette_file_opt = Some(format!("{}/{}", path, file_name));
                }
                filetypes::SkinAction::MeshFile(path, file_name) => {
                    println!("MeshFile: {}/{}", path, file_name);
                    mesh_file_opt = Some(format!("{}/{}", path, file_name));
                }
                filetypes::SkinAction::BackgroundColor(color_index) => {
                    println!("BackgroundColor: {}", color_index);
                }
                filetypes::SkinAction::SkinColor(face_index, color_index) => {
                    println!("SkinColor: face_index: {}, color_index: {}", face_index, color_index);
                }
            }
        }

        // todo: lots here

        Self {
            mesh_file: AssetDependency::Path(mesh_file_opt.unwrap()),
            palette_file: AssetDependency::Path(palette_file_opt.unwrap()),
        }
    }
}