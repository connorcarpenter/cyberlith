use naia_serde::{BitReader, Serde, SerdeErr, UnsignedVariableInteger};

use crate::{common::{SerdeRotation, VertexSerdeInt}, SkelAction};

impl SkelAction {
    pub fn read(bit_reader: &mut BitReader) -> Result<Vec<Self>, SerdeErr> {
        let mut output = Vec::new();

        // handle empty file
        if bit_reader.bytes_len() == 0 {
            return Ok(output);
        }

        // read loop
        loop {
            let continue_bool = bit_reader.read_bit()?;
            if !continue_bool {
                break;
            }

            // read X, Y, Z
            let x = VertexSerdeInt::de(bit_reader)?.to();
            let y = VertexSerdeInt::de(bit_reader)?.to();
            let z = VertexSerdeInt::de(bit_reader)?.to();
            let parent_id: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
            let parent_id_opt = {
                if parent_id == 0 {
                    None
                } else {
                    Some(parent_id - 1)
                }
            };
            let parent_and_angle_opt = if let Some(parent_id) = parent_id_opt {
                let angle = SerdeRotation::de(bit_reader)?;
                Some((parent_id, angle))
            } else {
                None
            };
            let vertex_name_opt = Option::<String>::de(bit_reader)?;
            let edge_name_opt = Option::<String>::de(bit_reader)?;

            output.push(Self::Vertex(
                x,
                y,
                z,
                parent_and_angle_opt,
                vertex_name_opt,
                edge_name_opt,
            ));
        }
        Ok(output)
    }
}