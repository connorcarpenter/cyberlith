use half::*;

use render_api::{
    base::{
        CubeSide, GeometryFunction, Interpolation, LightingModel, NormalDistributionFunction,
        Wrapping,
    },
    components::Viewport,
};

use crate::core::*;
use crate::renderer::RenderTargetExt;

///
/// Precalculations of light shining from an environment map (known as image based lighting - IBL).
/// This allows for real-time rendering of ambient light from the environment (see [AmbientLight](crate::AmbientLight)).
///
pub struct Environment {
    /// A cube map used to calculate the diffuse contribution from the environment.
    pub irradiance_map: GpuTextureCube,
    /// A cube map used to calculate the specular contribution from the environment.
    /// Each mip-map level contain the prefiltered color for a certain surface roughness.
    pub prefilter_map: GpuTextureCube,
    /// A 2D texture that contain the BRDF lookup tables (LUT).
    pub brdf_map: GpuTexture2D,
}

impl Environment {
    ///
    /// Computes the maps needed for physically based rendering with lighting from an environment from the given environment map.
    /// A default Cook-Torrance lighting model is used.
    ///
    pub fn new(environment_map: &GpuTextureCube) -> Self {
        Self::new_with_lighting_model(
            environment_map,
            LightingModel::Cook(
                NormalDistributionFunction::TrowbridgeReitzGGX,
                GeometryFunction::SmithSchlickGGX,
            ),
        )
    }

    ///
    /// Computes the maps needed for physically based rendering with lighting from an environment from the given environment map and with the specified lighting model.
    ///
    pub fn new_with_lighting_model(
        environment_map: &GpuTextureCube,
        lighting_model: LightingModel,
    ) -> Self {
        // Diffuse
        let irradiance_size = 32;
        let mut irradiance_map = GpuTextureCube::new_empty::<[f16; 4]>(
            irradiance_size,
            irradiance_size,
            Interpolation::Linear,
            Interpolation::Linear,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        {
            let fragment_shader_source = format!(
                "{}{}",
                include_str!("../../core/shared.frag"),
                include_str!("shaders/irradiance.frag")
            );
            let viewport = Viewport::new_at_origin(irradiance_size, irradiance_size);
            for side in CubeSide::iter() {
                irradiance_map
                    .as_color_target(&[side])
                    .clear(ClearState::default())
                    .write(|| {
                        apply_cube_effect(
                            side,
                            &fragment_shader_source,
                            RenderStates::default(),
                            viewport,
                            |program| {
                                program.use_texture_cube("environmentMap", environment_map);
                            },
                        )
                    });
            }
        }

        // Prefilter
        let prefilter_size = 128;
        let mut prefilter_map = GpuTextureCube::new_empty::<[f16; 4]>(
            prefilter_size,
            prefilter_size,
            Interpolation::Linear,
            Interpolation::Linear,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        {
            let fragment_shader_source = format!(
                "{}{}{}{}",
                super::lighting_model_shader(lighting_model),
                include_str!("../../core/shared.frag"),
                include_str!("shaders/light_shared.frag"),
                include_str!("shaders/prefilter.frag")
            );
            let max_mip_levels = 1;
            for mip in 0..max_mip_levels {
                for side in CubeSide::iter() {
                    let sides = [side];
                    let color_target = prefilter_map.as_color_target(&sides);
                    let viewport =
                        Viewport::new_at_origin(color_target.width(), color_target.height());
                    color_target.clear(ClearState::default()).write(|| {
                        apply_cube_effect(
                            side,
                            &fragment_shader_source,
                            RenderStates::default(),
                            viewport,
                            |program| {
                                program.use_texture_cube("environmentMap", environment_map);
                                program.use_uniform(
                                    "roughness",
                                    mip as f32 / (max_mip_levels as f32 - 1.0),
                                );
                                program.use_uniform("resolution", environment_map.width() as f32);
                            },
                        )
                    });
                }
            }
        }

        // BRDF
        let mut brdf_map = GpuTexture2D::new_empty::<[f32; 2]>(
            512,
            512,
            Interpolation::Linear,
            Interpolation::Linear,
            Wrapping::ClampToEdge,
            Wrapping::ClampToEdge,
        );
        let viewport = Viewport::new_at_origin(brdf_map.width(), brdf_map.height());
        brdf_map
            .as_color_target()
            .clear(ClearState::default())
            .write(|| {
                apply_effect(
                    &format!(
                        "{}{}{}{}",
                        super::lighting_model_shader(lighting_model),
                        include_str!("../../core/shared.frag"),
                        include_str!("shaders/light_shared.frag"),
                        include_str!("shaders/brdf.frag")
                    ),
                    RenderStates::default(),
                    viewport,
                    |_| {},
                )
            });

        Self {
            irradiance_map,
            prefilter_map,
            brdf_map,
        }
    }
}
