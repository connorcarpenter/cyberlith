use naia_serde::{BitReader, SerdeErr, SerdeInternal as Serde, UnsignedVariableInteger};

use crate::{common::VertexSerdeInt, mesh::MeshActionType, MeshAction};

impl MeshAction {
    pub fn read(bit_reader: &mut BitReader) -> Result<Vec<Self>, SerdeErr> {
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
                MeshActionType::Edge => {
                    let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    output.push(Self::Edge(vertex_a, vertex_b));
                }
                MeshActionType::Face => {
                    let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_c: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    let edge_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let edge_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let edge_c: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    output.push(Self::Face(
                        face_index, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c,
                    ));

                    face_index += 1;
                }
            }
        }
        Ok(output)
    }
}