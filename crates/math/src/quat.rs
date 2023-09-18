use glam::{Mat3, Quat, Vec3};

use naia_serde::{
    BitReader, BitWrite, ConstBitLength, SerdeErr, SerdeInternal as Serde, SignedInteger,
};

// Quat
pub fn quat_look_to(direction: Vec3, up: Vec3) -> Quat {
    let forward = direction.normalize();
    let right = up.cross(forward).normalize();
    let up = forward.cross(right);
    Quat::from_mat3(&Mat3::from_cols(right, up, forward))
}

// SerdeQuat
#[derive(Clone, Copy, PartialEq)]
pub struct SerdeQuat(Quat);

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
    use glam::Quat;
    use naia_serde::{ConstBitLength, Serde};
    use rand::Rng;

    use crate::SerdeQuat;

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
}
