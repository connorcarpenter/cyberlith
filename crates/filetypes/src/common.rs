use naia_serde::{BitReader, BitWrite, ConstBitLength, SerdeErr, SerdeInternal as Serde, SignedInteger, SignedVariableInteger, UnsignedInteger};

// NetTransformEntityType
#[derive(Serde, Copy, Clone, PartialEq, Debug)]
pub enum NetTransformEntityType {
    Uninit,
    Skin,
    Scene,
}

pub type VertexSerdeInt = SignedVariableInteger<4>;

#[derive(Serde, Copy, Clone, PartialEq, Debug)]
pub struct SerdeRotation {
    value: UnsignedInteger<6>,
}

impl SerdeRotation {
    const MAX_ANGLES: f32 = 64.0;
    const MAX_DEGREES: f32 = 360.0;

    pub fn from_degrees(degrees: f32) -> Self {
        let value_u8 = (degrees * Self::MAX_ANGLES / Self::MAX_DEGREES) as u8;
        let integer = UnsignedInteger::<6>::new(value_u8);

        Self { value: integer }
    }

    pub fn from_radians(radians: f32) -> Self {
        let degrees = f32::to_degrees(radians);
        Self::from_degrees(degrees)
    }

    // angle in degrees
    pub fn get_radians(&self) -> f32 {
        let degrees = self.get_degrees();
        f32::to_radians(degrees)
    }

    // angle in degrees
    pub fn set_radians(&mut self, value: f32) {
        let degrees = f32::to_degrees(value);
        self.set_degrees(degrees);
    }

    // angle in degrees
    pub fn get_degrees(&self) -> f32 {
        let value_u8: u8 = self.value.to();
        let value_f32 = value_u8 as f32;
        value_f32 * Self::MAX_DEGREES / Self::MAX_ANGLES
    }

    // angle in degrees
    pub fn set_degrees(&mut self, value_f32: f32) {
        let value_u8 = (value_f32 * Self::MAX_ANGLES / Self::MAX_DEGREES) as u8;
        let integer = UnsignedInteger::<6>::new(value_u8);
        self.value = integer;
    }
}

// SerdeQuat
#[derive(Clone, Copy, PartialEq)]
pub struct SerdeQuat {
    x: f32,
    y: f32,
    z: f32,
    w: f32,
}

impl SerdeQuat {
    const BITS: u8 = 5;
    const MAX_SIZE: f32 = 32.0;

    fn from_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
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

        let components = [self.x, self.y, self.z, self.w];
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
            SkipComponent::X => [self.y, self.z, self.w],
            SkipComponent::Y => [self.x, self.z, self.w],
            SkipComponent::Z => [self.x, self.y, self.w],
            SkipComponent::W => [self.x, self.y, self.z],
        };

        let skipped_is_negative = match skip_component {
            SkipComponent::X => self.x < 0.0,
            SkipComponent::Y => self.y < 0.0,
            SkipComponent::Z => self.z < 0.0,
            SkipComponent::W => self.w < 0.0,
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
                Self::from_xyzw(skipped_value, components[0], components[1], components[2])
            }
            SkipComponent::Y => {
                Self::from_xyzw(components[0], skipped_value, components[1], components[2])
            }
            SkipComponent::Z => {
                Self::from_xyzw(components[0], components[1], skipped_value, components[2])
            }
            SkipComponent::W => {
                Self::from_xyzw(components[0], components[1], components[2], skipped_value)
            }
        };

        Ok(quat)
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