use bevy_log::info;

use render_api::base::{Color, CpuMaterial};
use storage::{AssetHash, Storage, Handle};

impl AssetHash<PaletteData> for String {}

pub(crate) enum PaletteColor {
    Raw(u8, u8, u8),
    Material(Handle<CpuMaterial>),
}

pub struct PaletteData {
    colors: Vec<PaletteColor>,
}

impl PaletteData {
    pub(crate) fn get_cpu_mat_handle(&self, index: usize) -> Handle<CpuMaterial> {
        let PaletteColor::Material(handle) = &self.colors[index] else {
            panic!("expected material");
        };
        handle.clone()
    }
}

impl PaletteData {
    pub(crate) fn has_cpu_materials(&self) -> bool {
        if let Some(color) = self.colors.get(0) {
            if let PaletteColor::Material(_) = color {
                return true;
            }
        }
        return false;
    }

    pub(crate) fn load_cpu_materials(&mut self, materials: &mut Storage<CpuMaterial>) {
        for color in &mut self.colors {
            let PaletteColor::Raw(r, g, b) = color else {
                panic!("should only load once!");
            };
            let cpu_material_handle =
                materials.add(CpuMaterial::new(Color::new(*r, *g, *b), 0.0, 32.0, 0.5));
            *color = PaletteColor::Material(cpu_material_handle);
        }
    }
}

impl Default for PaletteData {
    fn default() -> Self {
        panic!("");
    }
}

impl From<String> for PaletteData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = web_fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let actions =
            asset_io::bits::PaletteAction::read(&data).expect("unable to parse file");

        info!("--- reading palette: {} ---", path);

        let mut colors = Vec::new();
        for action in actions {
            match action {
                asset_io::bits::PaletteAction::Color(r, g, b) => {
                    info!("loaded color {} : ({}, {}, {})", colors.len(), r, g, b);
                    colors.push(PaletteColor::Raw(r, g, b));
                }
            }
        }

        info!("--- done reading palette ---");

        Self { colors }
    }
}
