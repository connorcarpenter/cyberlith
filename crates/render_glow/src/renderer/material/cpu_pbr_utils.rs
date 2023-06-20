use half::f16;

use render_api::base::{CpuMaterial, TextureData};

///
/// Implement this for a [Material] that can be created from a [CpuMaterial].
///
pub trait FromPbrMaterial: std::marker::Sized {
    ///
    /// Creates a new material that can be used for rendering from a [CpuMaterial].
    ///
    fn from_cpu_material(cpu_material: &CpuMaterial) -> Self;
}

pub fn is_transparent(cpu_material: &CpuMaterial) -> bool {
    cpu_material.albedo.a != 255
        || cpu_material
        .albedo_texture
        .as_ref()
        .map(|t| match &t.initial_data() {
            Some(TextureData::RgbaU8(data)) => data.iter().any(|d| d[3] != 255),
            Some(TextureData::RgbaF16(data)) => data.iter().any(|d| d[3] < f16::from_f32(0.99)),
            Some(TextureData::RgbaF32(data)) => data.iter().any(|d| d[3] < 0.99),
            _ => false,
        })
            .unwrap_or(false)
}
