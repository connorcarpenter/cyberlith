use glam::{ Mat4, Quat, Vec3};

use naia_serde::{
    BitReader, BitWrite, ConstBitLength, SerdeErr, SerdeInternal as Serde, SignedInteger,
};

// SerdeQuat
#[derive(Clone, Copy, PartialEq)]
pub struct SerdeQuat(pub Quat);

impl From<Quat> for SerdeQuat {
    fn from(quat: Quat) -> Self {
        Self(quat)
    }
}

impl Into<Quat> for SerdeQuat {
    fn into(self) -> Quat {
        self.0
    }
}

impl SerdeQuat {
    const BITS: u8 = 5;
    const MAX_SIZE: f32 = 32.0;
}

#[derive(Serde, Clone, Copy, PartialEq)]
enum SkipComponent {
    X,
    Y,
    Z,
    W,
}

impl ConstBitLength for SkipComponent {
    fn const_bit_length() -> u32 {
        2
    }
}

impl Serde for SerdeQuat {
    fn ser(&self, writer: &mut dyn BitWrite) {
        let quat = self.0.normalize();

        let components = [quat.x, quat.y, quat.z, quat.w];
        let mut biggest_value = f32::MIN;
        let mut biggest_index: usize = 4;
        for (i, component) in components.iter().enumerate() {
            let component_abs = component.abs();
            if component_abs > biggest_value {
                biggest_value = component_abs;
                biggest_index = i;
            }
        }

        let skip_component = match biggest_index {
            0 => SkipComponent::X,
            1 => SkipComponent::Y,
            2 => SkipComponent::Z,
            3 => SkipComponent::W,
            _ => panic!("Invalid smallest index!"),
        };

        let components = match skip_component {
            SkipComponent::X => [quat.y, quat.z, quat.w],
            SkipComponent::Y => [quat.x, quat.z, quat.w],
            SkipComponent::Z => [quat.x, quat.y, quat.w],
            SkipComponent::W => [quat.x, quat.y, quat.z],
        };

        let skipped_is_negative = match skip_component {
            SkipComponent::X => quat.x < 0.0,
            SkipComponent::Y => quat.y < 0.0,
            SkipComponent::Z => quat.z < 0.0,
            SkipComponent::W => quat.w < 0.0,
        };

        // convert components to 6-bit signed integers
        let components = components.map(|component| {
            SignedInteger::<{ Self::BITS }>::new((component * Self::MAX_SIZE).round() as i128)
        });

        // serialize finally
        skip_component.ser(writer);
        skipped_is_negative.ser(writer);
        components.ser(writer);
    }

    fn de(reader: &mut BitReader) -> Result<Self, SerdeErr> {
        let skip_component = SkipComponent::de(reader)?;
        let skipped_is_negative = bool::de(reader)?;
        let components = <[SignedInteger<{ Self::BITS }>; 3]>::de(reader)?;

        // turn components back into f32s
        let components = components.map(|component| {
            let value: i128 = component.to();
            value as f32 / Self::MAX_SIZE
        });

        // get skipped value. noting that skipped value = sqrt(1 - a^2 + b^2 + c^2)
        let mut skipped_value =
            (1.0 - components[0].powi(2) - components[1].powi(2) - components[2].powi(2)).sqrt();

        // make negative if needed
        if skipped_is_negative {
            skipped_value = -skipped_value;
        }

        let quat = match skip_component {
            SkipComponent::X => {
                Quat::from_xyzw(skipped_value, components[0], components[1], components[2])
            }
            SkipComponent::Y => {
                Quat::from_xyzw(components[0], skipped_value, components[1], components[2])
            }
            SkipComponent::Z => {
                Quat::from_xyzw(components[0], components[1], skipped_value, components[2])
            }
            SkipComponent::W => {
                Quat::from_xyzw(components[0], components[1], components[2], skipped_value)
            }
        };

        Ok(Self(quat))
    }

    fn bit_length(&self) -> u32 {
        Self::const_bit_length()
    }
}

impl ConstBitLength for SerdeQuat {
    fn const_bit_length() -> u32 {
        SkipComponent::const_bit_length()
            + bool::const_bit_length()
            + <[SignedInteger<{ Self::BITS }>; 3]>::const_bit_length()
    }
}

#[cfg(test)]
mod tests {
    use glam::{Quat, Vec3};
    use naia_serde::{ConstBitLength, Serde};
    use rand::Rng;

    use crate::{quat_from_spin_direction, spin_direction_from_quat, SerdeQuat};

    #[test]
    fn identity() {
        let a = Quat::from_axis_angle(glam::Vec3::Z, f32::to_radians(0.0));
        let b = Quat::from_axis_angle(glam::Vec3::Z, f32::to_radians(5.0));

        assert!(!a.abs_diff_eq(b, 0.0001));
        assert!(a.abs_diff_eq(b, 0.1));
    }

    #[test]
    fn opposite() {
        let a = Quat::from_axis_angle(glam::Vec3::Z, f32::to_radians(0.0));
        let b = Quat::from_axis_angle(glam::Vec3::Z, f32::to_radians(180.0));

        assert!(!a.abs_diff_eq(b, 0.0001));

        println!("bits: {}", SerdeQuat::const_bit_length());
    }

    #[test]
    fn serde() {
        let mut count = 0;
        let mut rng = rand::thread_rng();

        loop {
            let x_rot = rng.gen_range(0.0..360.0);
            let y_rot = rng.gen_range(0.0..360.0);
            let z_rot = rng.gen_range(0.0..360.0);

            let quat_in = Quat::from_euler(
                glam::EulerRot::XYZ,
                f32::to_radians(x_rot),
                f32::to_radians(y_rot),
                f32::to_radians(z_rot),
            );

            // println!("Euler In: [{}, {}, {}]", x_rot, y_rot, z_rot);
            println!(
                "Quat In: [{}, {}, {}, {}]",
                quat_in.x, quat_in.y, quat_in.z, quat_in.w
            );

            let mut writer = naia_serde::BitWriter::new();
            SerdeQuat::from(quat_in).ser(&mut writer);

            let buffer = writer.to_bytes();

            let mut reader = naia_serde::BitReader::new(&buffer);
            let quat_out: Quat = SerdeQuat::de(&mut reader).unwrap().into();

            // let euler_out = quat_out.to_euler(glam::EulerRot::XYZ);
            // println!("Euler Out: [{}, {}, {}]",
            //          f32::to_degrees(euler_out.0),
            //          f32::to_degrees(euler_out.1),
            //          f32::to_degrees(euler_out.2));

            println!(
                "Quat Out: [{}, {}, {}, {}]",
                quat_out.x, quat_out.y, quat_out.z, quat_out.w
            );

            let angle_between = quat_in.angle_between(quat_out);
            println!("Diff: {}", angle_between);

            println!("---");

            assert!(angle_between < 0.1);
            assert!(quat_in.abs_diff_eq(quat_out, 0.04));

            count += 1;
            if count > 5000 {
                break;
            }
        }
    }

    #[test]
    fn quat_ops() {
        let mut rng = rand::thread_rng();

        let original_direction = Vec3::new(
            rng.gen_range(-10.0..10.0),
            rng.gen_range(-10.0..10.0),
            rng.gen_range(-10.0..10.0),
        )
        .normalize();

        println!("original_direction: {:?}", original_direction);

        let original_spin = 30.0; //rng.gen_range(0.0..360.0);

        println!("original_spin: {:?}", original_spin);

        let quat =
            quat_from_spin_direction(f32::to_radians(original_spin), Vec3::Z, original_direction);

        println!("quat: {:?}", quat);

        let (output_spin, output_direction) = spin_direction_from_quat(Vec3::Z, quat);
        let output_spin = output_spin.to_degrees();

        println!("output_direction: {:?}", output_direction);

        assert!(original_direction.abs_diff_eq(output_direction, 0.0001));

        println!("output_spin: {:?}", output_spin);

        let spin_diff = (original_spin - output_spin).abs();

        println!("spin_diff: {:?}", spin_diff);

        assert!(spin_diff < 0.0001);
    }
}

// spin is in radians
pub fn quat_from_spin_direction(spin: f32, base_direction: Vec3, target_direction: Vec3) -> Quat {
    let base_quat = Quat::from_axis_angle(
        base_direction.cross(target_direction).normalize(),
        base_direction.angle_between(target_direction),
    );
    let spin_quat = Quat::from_axis_angle(target_direction, spin);

    (spin_quat * base_quat).normalize()
}

pub fn spin_direction_from_quat(base_direction: Vec3, quat: Quat) -> (f32, Vec3) {
    let output_direction = (quat * base_direction).normalize();

    let output_spin: f32 = {
        let base_quat = Quat::from_axis_angle(
            base_direction.cross(output_direction).normalize(),
            base_direction.angle_between(output_direction),
        );
        angle_between_signed(quat, base_quat)
    };

    (output_spin, output_direction)
}

fn angle_between_signed(a: Quat, b: Quat) -> f32 {
    acos_approx(a.dot(b)) * 2.0
}

fn acos_approx(val: f32) -> f32 {
    // Based on https://github.com/microsoft/DirectXMath `XMScalarAcos`
    // Clamp input to [-1,1].
    let nonnegative = val >= 0.0;
    let x = val.abs();
    let mut omx = 1.0 - x;
    if omx < 0.0 {
        omx = 0.0;
    }
    let root = omx.sqrt();

    // 7-degree minimax approximation
    #[allow(clippy::approx_constant)]
    let mut result =
        ((((((-0.001_262_491_1 * x + 0.006_670_09) * x - 0.017_088_126) * x + 0.030_891_88) * x
            - 0.050_174_303)
            * x
            + 0.088_978_99)
            * x
            - 0.214_598_8)
            * x
            + 1.570_796_3;
    result *= root;

    // acos(x) = pi - acos(-x) when x < 0
    if nonnegative {
        result
    } else {
        core::f32::consts::PI - result
    }
}

pub fn matrix_transform_point(transform_mat: &Mat4, point: &Vec3) -> Vec3 {
    // Convert the point to a 4D vector (homogeneous coordinates)
    let mut point4 = point.extend(1.0);

    // Apply the transformation
    point4 = *transform_mat * point4;

    // Convert the result back to a 3D vector
    point4.truncate()
}
