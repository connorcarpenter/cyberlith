use naia_serde::{FileBitWriter, SerdeInternal as Serde, UnsignedVariableInteger};

use crate::{common::VertexSerdeInt, SkelAction};

impl SkelAction {
    pub fn write(actions: Vec<Self>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                SkelAction::Vertex(x, y, z, parent_id_opt, vertex_name_opt, edge_name_opt) => {
                    // continue bit
                    true.ser(&mut bit_writer);

                    // encode X, Y, Z
                    VertexSerdeInt::from(x).ser(&mut bit_writer);
                    VertexSerdeInt::from(y).ser(&mut bit_writer);
                    VertexSerdeInt::from(z).ser(&mut bit_writer);

                    // Parent Id
                    let parent_id = {
                        if let Some((parent_id, _)) = parent_id_opt {
                            parent_id + 1
                        } else {
                            0
                        }
                    };
                    UnsignedVariableInteger::<6>::from(parent_id).ser(&mut bit_writer);

                    // Angle
                    if let Some((_, angle)) = parent_id_opt {
                        angle.ser(&mut bit_writer);
                    }

                    // Names
                    vertex_name_opt.ser(&mut bit_writer);
                    edge_name_opt.ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        false.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}
