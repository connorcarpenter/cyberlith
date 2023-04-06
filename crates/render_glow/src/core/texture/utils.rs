use glow::HasContext;
use half::*;

use render_api::base::{Interpolation, Texture2D as CpuTexture, TextureData, Wrapping};

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
    mip_map_filter: Option<Interpolation>,
    wrap_s: Wrapping,
    wrap_t: Wrapping,
    wrap_r: Option<Wrapping>,
) {
    unsafe {
        let context = Context::get();
        match mip_map_filter {
            None => context.tex_parameter_i32(
                target,
                glow::TEXTURE_MIN_FILTER,
                interpolation_from(min_filter),
            ),
            Some(Interpolation::Nearest) => {
                if min_filter == Interpolation::Nearest {
                    context.tex_parameter_i32(
                        target,
                        glow::TEXTURE_MIN_FILTER,
                        glow::NEAREST_MIPMAP_NEAREST as i32,
                    );
                } else {
                    context.tex_parameter_i32(
                        target,
                        glow::TEXTURE_MIN_FILTER,
                        glow::LINEAR_MIPMAP_NEAREST as i32,
                    )
                }
            }
            Some(Interpolation::Linear) => {
                if min_filter == Interpolation::Nearest {
                    context.tex_parameter_i32(
                        target,
                        glow::TEXTURE_MIN_FILTER,
                        glow::NEAREST_MIPMAP_LINEAR as i32,
                    );
                } else {
                    context.tex_parameter_i32(
                        target,
                        glow::TEXTURE_MIN_FILTER,
                        glow::LINEAR_MIPMAP_LINEAR as i32,
                    )
                }
            }
            _ => panic!("Can only sample textures using 'NEAREST' or 'LINEAR' interpolation"),
        }
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

pub fn calculate_number_of_mip_maps(
    mip_map_filter: Option<Interpolation>,
    width: u32,
    height: u32,
    depth: Option<u32>,
) -> u32 {
    if mip_map_filter.is_some()
        && width == height
        && depth.map(|d| d == width).unwrap_or(true)
        && width.is_power_of_two()
    {
        (width as f64).log2() as u32 + 1
    } else {
        1
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

pub fn ru8_data(t: &CpuTexture) -> &[u8] {
    if let TextureData::RU8(data) = &t.data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgu8_data(t: &CpuTexture) -> &[[u8; 2]] {
    if let TextureData::RgU8(data) = &t.data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgbu8_data(t: &CpuTexture) -> &[[u8; 3]] {
    if let TextureData::RgbU8(data) = &t.data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgbau8_data(t: &CpuTexture) -> &[[u8; 4]] {
    if let TextureData::RgbaU8(data) = &t.data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rf16_data(t: &CpuTexture) -> &[f16] {
    if let TextureData::RF16(data) = &t.data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgf16_data(t: &CpuTexture) -> &[[f16; 2]] {
    if let TextureData::RgF16(data) = &t.data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgbf16_data(t: &CpuTexture) -> &[[f16; 3]] {
    if let TextureData::RgbF16(data) = &t.data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgbaf16_data(t: &CpuTexture) -> &[[f16; 4]] {
    if let TextureData::RgbaF16(data) = &t.data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rf32_data(t: &CpuTexture) -> &[f32] {
    if let TextureData::RF32(data) = &t.data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgf32_data(t: &CpuTexture) -> &[[f32; 2]] {
    if let TextureData::RgF32(data) = &t.data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgbf32_data(t: &CpuTexture) -> &[[f32; 3]] {
    if let TextureData::RgbF32(data) = &t.data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}

pub fn rgbaf32_data(t: &CpuTexture) -> &[[f32; 4]] {
    if let TextureData::RgbaF32(data) = &t.data() {
        data
    } else {
        panic!("all of the images used for cube map sides must have the same texture data type")
    }
}
