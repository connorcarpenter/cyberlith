use naia_serde::{BitReader, SerdeErr, SerdeInternal as Serde, UnsignedVariableInteger};

use crate::bits::{common::VertexSerdeInt, mesh::MeshActionType, MeshAction};

impl MeshAction {
    pub fn read(bytes: &[u8]) -> Result<Vec<Self>, SerdeErr> {
        let mut bit_reader = BitReader::new(bytes);
        let bit_reader = &mut bit_reader;
        let mut output = Vec::new();

        // handle empty file
        if bit_reader.bytes_len() == 0 {
            return Ok(output);
        }

        let mut face_index = 0;

        // read loop
        loop {
            let continue_type = MeshActionType::de(bit_reader)?;

            match continue_type {
                MeshActionType::None => break,
                MeshActionType::Vertex => {
                    // read X, Y, Z
                    let x = VertexSerdeInt::de(bit_reader)?.to();
                    let y = VertexSerdeInt::de(bit_reader)?.to();
                    let z = VertexSerdeInt::de(bit_reader)?.to();

                    output.push(Self::Vertex(x, y, z));
                }
                MeshActionType::Face => {
                    let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_c: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    output.push(Self::Face(face_index, vertex_a, vertex_b, vertex_c));

                    face_index += 1;
                }
            }
        }
        Ok(output)
    }
}
