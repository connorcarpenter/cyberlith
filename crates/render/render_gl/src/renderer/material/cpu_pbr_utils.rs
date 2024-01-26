use render_api::base::CpuMaterial;

///
/// Implement this for a [Material] that can be created from a [CpuMaterial].
///
pub trait FromPbrMaterial: Sized {
    ///
    /// Creates a new material that can be used for rendering from a [CpuMaterial].
    ///
    fn from_cpu_material(cpu_material: &CpuMaterial) -> Self;
}
