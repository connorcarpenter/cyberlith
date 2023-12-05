use std::fs;

use naia_serde::BitReader;

use render_api::{AssetHash, Handle, base::{Color, CpuMaterial}, Assets};

impl AssetHash<PaletteData> for String {}

#[derive(Debug)]
pub(crate) enum PaletteColor {
    Raw(u8, u8, u8),
    Material(Handle<CpuMaterial>),
}

#[derive(Debug)]
pub struct PaletteData {
    colors: Vec<PaletteColor>,
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

    pub(crate) fn load_cpu_materials(&mut self, materials: &mut Assets<CpuMaterial>) {
        for color in &mut self.colors {
            let PaletteColor::Raw(r, g, b) = color else {
                panic!("should only load once!");
            };
            let cpu_material_handle = materials.add(CpuMaterial::new(Color::new(*r, *g, *b), 0.0, 0.0, 0.0));
            *color = PaletteColor::Material(cpu_material_handle);
        }
    }

    pub(crate) fn get_cpu_material_handles(&self) -> Vec<Handle<CpuMaterial>> {
        self.colors.iter().map(|color| {
            if let PaletteColor::Material(handle) = color {
                *handle
            } else {
                panic!("should only load once!");
            }
        }).collect::<Vec<_>>()
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

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::PaletteAction::read(&mut bit_reader).expect("unable to parse file");

        let mut colors = Vec::new();
        for action in actions {
            match action {
                filetypes::PaletteAction::Color(r, g, b) => {
                    println!("Color: ({}, {}, {})", r, g, b);
                    colors.push(PaletteColor::Raw(r, g, b));
                }
            }
        }

        Self {
            colors
        }
    }
}