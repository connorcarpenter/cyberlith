use gl::{HasContext, UniformLocation};

use math::*;
use render_api::base::*;

use crate::core::*;

pub enum UniformType {
    Value,
    Vec2,
    Vec3,
    Vec4,
    Mat2,
    Mat3,
    Mat4,
}

pub trait PrimitiveDataType: DataType + Copy + Default {
    fn send_uniform_with_type(location: &UniformLocation, data: &[Self], type_: UniformType);
    fn internal_format_with_size(size: u32) -> u32;
}

impl PrimitiveDataType for u8 {
    fn internal_format_with_size(size: u32) -> u32 {
        match size {
            1 => gl::R8,
            2 => gl::RG8,
            3 => gl::RGB8,
            4 => gl::RGBA8,
            _ => unreachable!(),
        }
    }

    fn send_uniform_with_type(location: &UniformLocation, data: &[Self], type_: UniformType) {
        let data = data.iter().map(|v| *v as u32).collect::<Vec<_>>();
        u32::send_uniform_with_type(location, &data, type_)
    }
}
impl PrimitiveDataType for u16 {
    fn internal_format_with_size(size: u32) -> u32 {
        match size {
            1 => gl::R16UI,
            2 => gl::RG16UI,
            3 => gl::RGB16UI,
            4 => gl::RGBA16UI,
            _ => unreachable!(),
        }
    }

    fn send_uniform_with_type(location: &UniformLocation, data: &[Self], type_: UniformType) {
        let data = data.iter().map(|v| *v as u32).collect::<Vec<_>>();
        u32::send_uniform_with_type(location, &data, type_)
    }
}
impl PrimitiveDataType for u32 {
    fn internal_format_with_size(size: u32) -> u32 {
        match size {
            1 => gl::R32UI,
            2 => gl::RG32UI,
            3 => gl::RGB32UI,
            4 => gl::RGBA32UI,
            _ => unreachable!(),
        }
    }

    fn send_uniform_with_type(location: &UniformLocation, data: &[Self], type_: UniformType) {
        unsafe {
            let context = Context::get();
            match type_ {
                UniformType::Value => context.uniform_1_u32_slice(Some(location), data),
                UniformType::Vec2 => context.uniform_2_u32_slice(Some(location), data),
                UniformType::Vec3 => context.uniform_3_u32_slice(Some(location), data),
                UniformType::Vec4 => context.uniform_4_u32_slice(Some(location), data),
                _ => unimplemented!(),
            }
        }
    }
}
impl PrimitiveDataType for i8 {
    fn internal_format_with_size(size: u32) -> u32 {
        match size {
            1 => gl::R8I,
            2 => gl::RG8I,
            3 => gl::RGB8I,
            4 => gl::RGBA8I,
            _ => unreachable!(),
        }
    }

    fn send_uniform_with_type(location: &UniformLocation, data: &[Self], type_: UniformType) {
        let data = data.iter().map(|v| *v as i32).collect::<Vec<_>>();
        i32::send_uniform_with_type(location, &data, type_)
    }
}
impl PrimitiveDataType for i16 {
    fn internal_format_with_size(size: u32) -> u32 {
        match size {
            1 => gl::R16I,
            2 => gl::RG16I,
            3 => gl::RGB16I,
            4 => gl::RGBA16I,
            _ => unreachable!(),
        }
    }

    fn send_uniform_with_type(location: &UniformLocation, data: &[Self], type_: UniformType) {
        let data = data.iter().map(|v| *v as i32).collect::<Vec<_>>();
        i32::send_uniform_with_type(location, &data, type_)
    }
}
impl PrimitiveDataType for i32 {
    fn internal_format_with_size(size: u32) -> u32 {
        match size {
            1 => gl::R32I,
            2 => gl::RG32I,
            3 => gl::RGB32I,
            4 => gl::RGBA32I,
            _ => unreachable!(),
        }
    }

    fn send_uniform_with_type(location: &UniformLocation, data: &[Self], type_: UniformType) {
        unsafe {
            let context = Context::get();
            match type_ {
                UniformType::Value => context.uniform_1_i32_slice(Some(location), data),
                UniformType::Vec2 => context.uniform_2_i32_slice(Some(location), data),
                UniformType::Vec3 => context.uniform_3_i32_slice(Some(location), data),
                UniformType::Vec4 => context.uniform_4_i32_slice(Some(location), data),
                _ => unimplemented!(),
            }
        }
    }
}

impl PrimitiveDataType for f32 {
    fn internal_format_with_size(size: u32) -> u32 {
        match size {
            1 => gl::R32F,
            2 => gl::RG32F,
            3 => gl::RGB32F,
            4 => gl::RGBA32F,
            _ => unreachable!(),
        }
    }

    fn send_uniform_with_type(location: &UniformLocation, data: &[Self], type_: UniformType) {
        unsafe {
            let context = Context::get();
            match type_ {
                UniformType::Value => context.uniform_1_f32_slice(Some(location), data),
                UniformType::Vec2 => context.uniform_2_f32_slice(Some(location), data),
                UniformType::Vec3 => context.uniform_3_f32_slice(Some(location), data),
                UniformType::Vec4 => context.uniform_4_f32_slice(Some(location), data),
                UniformType::Mat2 => {
                    context.uniform_matrix_2_f32_slice(Some(location), false, data)
                }
                UniformType::Mat3 => {
                    context.uniform_matrix_3_f32_slice(Some(location), false, data)
                }
                UniformType::Mat4 => {
                    context.uniform_matrix_4_f32_slice(Some(location), false, data)
                }
            }
        }
    }
}

pub trait DataType: std::fmt::Debug + Clone {
    fn internal_format() -> u32;
    fn data_type() -> u32;
    fn size() -> u32;
    fn send_uniform(location: &UniformLocation, data: &[Self]);
}

impl DataType for u8 {
    fn internal_format() -> u32 {
        Self::internal_format_with_size(1)
    }

    fn data_type() -> u32 {
        gl::UNSIGNED_BYTE
    }

    fn size() -> u32 {
        1
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        Self::send_uniform_with_type(location, data, UniformType::Value)
    }
}

impl DataType for u16 {
    fn internal_format() -> u32 {
        Self::internal_format_with_size(1)
    }
    fn data_type() -> u32 {
        gl::UNSIGNED_SHORT
    }

    fn size() -> u32 {
        1
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        Self::send_uniform_with_type(location, data, UniformType::Value)
    }
}

impl DataType for u32 {
    fn internal_format() -> u32 {
        Self::internal_format_with_size(1)
    }

    fn data_type() -> u32 {
        gl::UNSIGNED_INT
    }

    fn size() -> u32 {
        1
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        Self::send_uniform_with_type(location, data, UniformType::Value)
    }
}

impl DataType for i8 {
    fn internal_format() -> u32 {
        Self::internal_format_with_size(1)
    }

    fn data_type() -> u32 {
        gl::BYTE
    }

    fn size() -> u32 {
        1
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        Self::send_uniform_with_type(location, data, UniformType::Value)
    }
}

impl DataType for i16 {
    fn internal_format() -> u32 {
        Self::internal_format_with_size(1)
    }

    fn data_type() -> u32 {
        gl::SHORT
    }

    fn size() -> u32 {
        1
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        Self::send_uniform_with_type(location, data, UniformType::Value)
    }
}

impl DataType for i32 {
    fn internal_format() -> u32 {
        Self::internal_format_with_size(1)
    }

    fn data_type() -> u32 {
        gl::INT
    }

    fn size() -> u32 {
        1
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        Self::send_uniform_with_type(location, data, UniformType::Value)
    }
}

impl DataType for f32 {
    fn internal_format() -> u32 {
        Self::internal_format_with_size(1)
    }

    fn data_type() -> u32 {
        gl::FLOAT
    }

    fn size() -> u32 {
        1
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        Self::send_uniform_with_type(location, data, UniformType::Value)
    }
}

impl DataType for Vec2 {
    fn internal_format() -> u32 {
        f32::internal_format_with_size(Self::size())
    }

    fn data_type() -> u32 {
        f32::data_type()
    }

    fn size() -> u32 {
        2
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        let data = data.iter().flat_map(|v| [v.x, v.y]).collect::<Vec<_>>();
        f32::send_uniform_with_type(location, &data, UniformType::Vec2)
    }
}

impl<T: PrimitiveDataType> DataType for [T; 2] {
    fn internal_format() -> u32 {
        T::internal_format_with_size(Self::size())
    }

    fn data_type() -> u32 {
        T::data_type()
    }

    fn size() -> u32 {
        2
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        let data = data.iter().flatten().copied().collect::<Vec<_>>();
        T::send_uniform_with_type(location, &data, UniformType::Vec2)
    }
}

impl DataType for Vec3 {
    fn internal_format() -> u32 {
        f32::internal_format_with_size(Self::size())
    }
    fn data_type() -> u32 {
        f32::data_type()
    }

    fn size() -> u32 {
        3
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        let data = data
            .iter()
            .flat_map(|v| [v.x, v.y, v.z])
            .collect::<Vec<_>>();
        f32::send_uniform_with_type(location, &data, UniformType::Vec3)
    }
}

impl<T: PrimitiveDataType> DataType for [T; 3] {
    fn internal_format() -> u32 {
        T::internal_format_with_size(Self::size())
    }
    fn data_type() -> u32 {
        T::data_type()
    }

    fn size() -> u32 {
        3
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        let data = data.iter().flatten().copied().collect::<Vec<_>>();
        T::send_uniform_with_type(location, &data, UniformType::Vec3)
    }
}

impl DataType for Vec4 {
    fn internal_format() -> u32 {
        f32::internal_format_with_size(Self::size())
    }

    fn data_type() -> u32 {
        f32::data_type()
    }

    fn size() -> u32 {
        4
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        let data = data
            .iter()
            .flat_map(|v| [v.x, v.y, v.z, v.w])
            .collect::<Vec<_>>();
        f32::send_uniform_with_type(location, &data, UniformType::Vec4)
    }
}

impl<T: PrimitiveDataType> DataType for [T; 4] {
    fn internal_format() -> u32 {
        T::internal_format_with_size(Self::size())
    }

    fn data_type() -> u32 {
        T::data_type()
    }

    fn size() -> u32 {
        4
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        let data = data.iter().flatten().copied().collect::<Vec<_>>();
        T::send_uniform_with_type(location, &data, UniformType::Vec4)
    }
}

impl DataType for Quat {
    fn internal_format() -> u32 {
        f32::internal_format_with_size(Self::size())
    }

    fn data_type() -> u32 {
        f32::data_type()
    }

    fn size() -> u32 {
        4
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        let data = data
            .iter()
            .flat_map(|v| [v.x, v.y, v.z, v.w])
            .collect::<Vec<_>>();
        f32::send_uniform_with_type(location, &data, UniformType::Vec4)
    }
}

impl DataType for Color {
    fn internal_format() -> u32 {
        u8::internal_format_with_size(Self::size())
    }

    fn data_type() -> u32 {
        u8::data_type()
    }

    fn size() -> u32 {
        4
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        let data = data
            .iter()
            .flat_map(|v| [v.r as f32 / 255.0, v.g as f32 / 255.0, v.b as f32 / 255.0])
            .collect::<Vec<_>>();
        f32::send_uniform_with_type(location, &data, UniformType::Vec3)
    }
}

impl DataType for Mat2 {
    fn internal_format() -> u32 {
        f32::internal_format_with_size(Self::size())
    }

    fn data_type() -> u32 {
        f32::data_type()
    }

    fn size() -> u32 {
        4
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        let data = data
            .iter()
            .flat_map(|v| [v.x_axis.x, v.x_axis.y, v.y_axis.x, v.y_axis.y])
            .collect::<Vec<_>>();
        f32::send_uniform_with_type(location, &data, UniformType::Mat2)
    }
}

impl DataType for Mat3 {
    fn internal_format() -> u32 {
        f32::internal_format_with_size(Self::size())
    }

    fn data_type() -> u32 {
        f32::data_type()
    }

    fn size() -> u32 {
        9
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        let data = data
            .iter()
            .flat_map(|v| {
                [
                    v.x_axis.x, v.x_axis.y, v.x_axis.z, v.y_axis.x, v.y_axis.y, v.y_axis.z,
                    v.z_axis.x, v.z_axis.y, v.z_axis.z,
                ]
            })
            .collect::<Vec<_>>();
        f32::send_uniform_with_type(location, &data, UniformType::Mat3)
    }
}

impl DataType for Mat4 {
    fn internal_format() -> u32 {
        f32::internal_format_with_size(Self::size())
    }

    fn data_type() -> u32 {
        f32::data_type()
    }

    fn size() -> u32 {
        16
    }

    fn send_uniform(location: &UniformLocation, data: &[Self]) {
        let data = data
            .iter()
            .flat_map(|v| {
                [
                    v.x_axis.x, v.x_axis.y, v.x_axis.z, v.x_axis.w, v.y_axis.x, v.y_axis.y,
                    v.y_axis.z, v.y_axis.w, v.z_axis.x, v.z_axis.y, v.z_axis.z, v.z_axis.w,
                    v.w_axis.x, v.w_axis.y, v.w_axis.z, v.w_axis.w,
                ]
            })
            .collect::<Vec<_>>();
        f32::send_uniform_with_type(location, &data, UniformType::Mat4)
    }
}

pub trait DepthDataType {
    fn internal_format() -> u32;
}

impl DepthDataType for f32 {
    fn internal_format() -> u32 {
        gl::DEPTH_COMPONENT32F
    }
}
