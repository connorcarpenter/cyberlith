use naia_serde::{BitReader, FileBitWriter, Serde, SerdeErr, UnsignedVariableInteger};

use crate::common::{SerdeRotation, VertexSerdeInt};

// Actions
#[derive(Debug)]
enum SkelAction {
    //////// x,   y,   z, Option<parent_id, angle>, vertex_name, edge_name //
    Vertex(
        i16,
        i16,
        i16,
        Option<(u16, SerdeRotation)>,
        Option<String>,
        Option<String>,
    ),
}

// Writer
pub struct SkelWriter;

impl SkelWriter {

    fn write_from_actions(&self, actions: Vec<SkelAction>) -> Box<[u8]> {
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

// Reader
pub struct SkelReader;

impl SkelReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<SkelAction>, SerdeErr> {
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

            output.push(SkelAction::Vertex(
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