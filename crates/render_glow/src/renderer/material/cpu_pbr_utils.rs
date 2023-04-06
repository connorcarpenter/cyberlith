use half::f16;

use render_api::base::{PbrMaterial, TextureData};

///
/// Implement this for a [Material] that can be created from a [PbrMaterial].
///
pub trait FromPbrMaterial: std::marker::Sized {
    ///
    /// Creates a new material that can be used for rendering from a [PbrMaterial].
    ///
    fn from_cpu_material(cpu_material: &PbrMaterial) -> Self;
}

pub fn is_transparent(cpu_material: &PbrMaterial) -> bool {
    cpu_material.albedo.a != 255
        || cpu_material
            .albedo_texture
            .as_ref()
            .map(|t| match &t.data() {
                TextureData::RgbaU8(data) => data.iter().any(|d| d[3] != 255),
                TextureData::RgbaF16(data) => data.iter().any(|d| d[3] < f16::from_f32(0.99)),
                TextureData::RgbaF32(data) => data.iter().any(|d| d[3] < 0.99),
                _ => false,
            })
            .unwrap_or(false)
}
