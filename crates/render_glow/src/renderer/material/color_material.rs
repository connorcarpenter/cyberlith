use std::sync::Arc;

use render_api::base::{Camera, Color, PbrMaterial};

use crate::{core::*, renderer::*};

///
/// A material that renders a [Geometry] in a color defined by multiplying a color with an optional texture and optional per vertex colors.
/// This material is not affected by lights.
///
#[derive(Clone, Default)]
pub struct ColorMaterial {
    /// Base surface color. Assumed to be in linear color space.
    pub color: Color,
    /// An optional texture which is samples using uv coordinates (requires that the [Geometry] supports uv coordinates).
    pub texture: Option<Texture2DRef>,
    /// Render states.
    pub render_states: RenderStates,
    /// Whether this material should be treated as a transparent material (An object needs to be rendered differently depending on whether it is transparent or opaque).
    pub is_transparent: bool,
}

impl ColorMaterial {
    ///
    /// Constructs a new color material from a [PbrMaterial].
    /// Tries to infer whether this material is transparent or opaque from the alpha value of the albedo color and the alpha values in the albedo texture.
    /// Since this is not always correct, it is preferred to use [ColorMaterial::new_opaque] or [ColorMaterial::new_transparent].
    ///
    pub fn new(cpu_material: &PbrMaterial) -> Self {
        if is_transparent(cpu_material) {
            Self::new_transparent(cpu_material)
        } else {
            Self::new_opaque(cpu_material)
        }
    }

    /// Constructs a new opaque color material from a [PbrMaterial].
    pub fn new_opaque(cpu_material: &PbrMaterial) -> Self {
        let texture = cpu_material
            .albedo_texture
            .as_ref()
            .map(|cpu_texture| Arc::new(Texture2D::new(cpu_texture)).into());
        Self {
            color: cpu_material.albedo,
            texture,
            is_transparent: false,
            render_states: RenderStates::default(),
        }
    }

    /// Constructs a new transparent color material from a [PbrMaterial].
    pub fn new_transparent(cpu_material: &PbrMaterial) -> Self {
        let texture = cpu_material
            .albedo_texture
            .as_ref()
            .map(|cpu_texture| Arc::new(Texture2D::new(cpu_texture)).into());
        Self {
            color: cpu_material.albedo,
            texture,
            is_transparent: true,
            render_states: RenderStates {
                write_mask: WriteMask::COLOR,
                blend: Blend::TRANSPARENCY,
                ..Default::default()
            },
        }
    }

    /// Creates a color material from a [PhysicalMaterial].
    pub fn from_physical_material(physical_material: &PhysicalMaterial) -> Self {
        Self {
            color: physical_material.albedo,
            texture: physical_material.albedo_texture.clone(),
            render_states: physical_material.render_states,
            is_transparent: physical_material.is_transparent,
        }
    }
}

impl FromPbrMaterial for ColorMaterial {
    fn from_cpu_material(cpu_material: &PbrMaterial) -> Self {
        Self::new(cpu_material)
    }
}

impl Material for ColorMaterial {
    fn fragment_shader(&self, _lights: &[&dyn Light]) -> FragmentShader {
        let mut attributes = FragmentAttributes {
            color: true,
            ..FragmentAttributes::NONE
        };
        let mut shader = String::new();
        if self.texture.is_some() {
            attributes.uv = true;
            shader.push_str("#define USE_TEXTURE\nin vec2 uvs;\n");
        }
        shader.push_str(include_str!("../../core/shared.frag"));
        shader.push_str(include_str!("shaders/color_material.frag"));
        FragmentShader {
            source: shader,
            attributes,
        }
    }

    fn use_uniforms(&self, program: &Program, _camera: &Camera, _lights: &[&dyn Light]) {
        program.use_uniform("surfaceColor", self.color);
        if let Some(ref tex) = self.texture {
            program.use_uniform("textureTransformation", tex.transformation);
            program.use_texture("tex", tex);
        }
    }
    fn render_states(&self) -> RenderStates {
        self.render_states
    }
    fn material_type(&self) -> MaterialType {
        if self.is_transparent {
            MaterialType::Transparent
        } else {
            MaterialType::Opaque
        }
    }
}
