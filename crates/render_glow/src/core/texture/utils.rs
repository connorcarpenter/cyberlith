use glow::HasContext;
use half::*;

use render_api::base::{CpuTexture2D, CpuTextureData, Interpolation, Wrapping};

use crate::core::*;

// COMMON TEXTURE FUNCTIONS

pub fn generate() -> glow::Texture {
    unsafe {
        Context::get()
            .create_texture()
            .expect("Failed creating texture")
    }
}

pub fn set_parameters(
    target: u32,
    min_filter: Interpolation,
    mag_filter: Interpolation,
    wrap_s: Wrapping,
    wrap_t: Wrapping,
    wrap_r: Option<Wrapping>,
) {
    unsafe {
        let context = Context::get();
        context.tex_parameter_i32(
            target,
            glow::TEXTURE_MIN_FILTER,
            interpolation_from(min_filter),
        );
        context.tex_parameter_i32(
            target,
            glow::TEXTURE_MAG_FILTER,
            interpolation_from(mag_filter),
        );
        context.tex_parameter_i32(target, glow::TEXTURE_WRAP_S, wrapping_from(wrap_s));
        context.tex_parameter_i32(target, glow::TEXTURE_WRAP_T, wrapping_from(wrap_t));
        if let Some(r) = wrap_r {
            context.tex_parameter_i32(target, glow::TEXTURE_WRAP_R, wrapping_from(r));
        }
    }
}

fn wrapping_from(wrapping: Wrapping) -> i32 {
    (match wrapping {
        Wrapping::Repeat => glow::REPEAT,
        Wrapping::MirroredRepeat => glow::MIRRORED_REPEAT,
        Wrapping::ClampToEdge => glow::CLAMP_TO_EDGE,
    }) as i32
}

fn interpolation_from(interpolation: Interpolation) -> i32 {
    (match interpolation {
        Interpolation::Nearest => glow::NEAREST,
        Interpolation::Linear => glow::LINEAR,
        _ => panic!("Can only sample textures using 'NEAREST' or 'LINEAR' interpolation"),
    }) as i32
}

pub fn check_data_length<T: TextureDataType>(
    width: u32,
    height: u32,
    depth: u32,
    data_byte_size: usize,
    data_len: usize,
) {
    let expected_bytes = width as usize * height as usize * depth as usize * data_byte_size;
    let actual_bytes = data_len * std::mem::size_of::<T>();
    if expected_bytes != actual_bytes {
        panic!(
            "invalid size of texture data (expected {} bytes but got {} bytes)",
            expected_bytes, actual_bytes
        )
    }
}

pub fn ru8_data(t: &CpuTexture2D) -> &[u8] {
    if let Some(CpuTextureData::RU8(data)) = &t.initial_data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgu8_data(t: &CpuTexture2D) -> &[[u8; 2]] {
    if let Some(CpuTextureData::RgU8(data)) = &t.initial_data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgbu8_data(t: &CpuTexture2D) -> &[[u8; 3]] {
    if let Some(CpuTextureData::RgbU8(data)) = &t.initial_data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgbau8_data(t: &CpuTexture2D) -> &[[u8; 4]] {
    if let Some(CpuTextureData::RgbaU8(data)) = &t.initial_data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rf16_data(t: &CpuTexture2D) -> &[f16] {
    if let Some(CpuTextureData::RF16(data)) = &t.initial_data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgf16_data(t: &CpuTexture2D) -> &[[f16; 2]] {
    if let Some(CpuTextureData::RgF16(data)) = &t.initial_data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgbf16_data(t: &CpuTexture2D) -> &[[f16; 3]] {
    if let Some(CpuTextureData::RgbF16(data)) = &t.initial_data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgbaf16_data(t: &CpuTexture2D) -> &[[f16; 4]] {
    if let Some(CpuTextureData::RgbaF16(data)) = &t.initial_data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rf32_data(t: &CpuTexture2D) -> &[f32] {
    if let Some(CpuTextureData::RF32(data)) = &t.initial_data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgf32_data(t: &CpuTexture2D) -> &[[f32; 2]] {
    if let Some(CpuTextureData::RgF32(data)) = &t.initial_data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgbf32_data(t: &CpuTexture2D) -> &[[f32; 3]] {
    if let Some(CpuTextureData::RgbF32(data)) = &t.initial_data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgbaf32_data(t: &CpuTexture2D) -> &[[f32; 4]] {
    if let Some(CpuTextureData::RgbaF32(data)) = &t.initial_data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}
