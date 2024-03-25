
use render_api::base::{Color, CpuMaterial};
use storage::{Handle, Storage};

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

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let actions = asset_io::bits::PaletteAction::read(bytes).expect("unable to parse file");

        // info!("--- reading palette ---");

        let mut colors = Vec::new();
        for action in actions {
            match action {
                asset_io::bits::PaletteAction::Color(r, g, b) => {
                    // info!("loaded color {} : ({}, {}, {})", colors.len(), r, g, b);
                    colors.push(PaletteColor::Raw(r, g, b));
                }
            }
        }

        // info!("--- done reading palette ---");

        Self { colors }
    }
}

impl Default for PaletteData {
    fn default() -> Self {
        panic!("");
    }
}
