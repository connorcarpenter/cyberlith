use naia_serde::{BitReader, SerdeErr, SerdeInternal as Serde, UnsignedVariableInteger};

use crate::bits::{
    common::VertexSerdeInt,
    icon::{IconActionType, IconFrameAction, IconFrameActionType},
    IconAction,
};

impl IconAction {
    pub fn read(bytes: &[u8]) -> Result<Vec<Self>, SerdeErr> {
        let mut bit_reader = BitReader::new(bytes);
        let bit_reader = &mut bit_reader;
        let mut output = Vec::new();

        // handle empty file
        if bit_reader.bytes_len() == 0 {
            return Ok(output);
        }

        // read loop
        'outer: loop {
            let action_type = IconActionType::de(bit_reader)?;

            match action_type {
                IconActionType::None => break 'outer,
                IconActionType::PaletteFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    output.push(Self::PaletteFile(path, file_name));
                }
                IconActionType::Frame => {
                    let mut face_index = 0;

                    let mut frame_output = Vec::new();

                    'inner: loop {
                        let frame_action_type = IconFrameActionType::de(bit_reader)?;

                        match frame_action_type {
                            IconFrameActionType::None => break 'inner,
                            IconFrameActionType::Vertex => {
                                // read X, Y
                                let x = VertexSerdeInt::de(bit_reader)?.to();
                                let y = VertexSerdeInt::de(bit_reader)?.to();

                                frame_output.push(IconFrameAction::Vertex(x, y));
                            }
                            IconFrameActionType::Edge => {
                                let vertex_a: u16 =
                                    UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let vertex_b: u16 =
                                    UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                                frame_output.push(IconFrameAction::Edge(vertex_a, vertex_b));
                            }
                            IconFrameActionType::Face => {
                                let palette_color_index = u8::de(bit_reader)?;

                                let vertex_a: u16 =
                                    UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let vertex_b: u16 =
                                    UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let vertex_c: u16 =
                                    UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                                let edge_a: u16 =
                                    UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let edge_b: u16 =
                                    UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let edge_c: u16 =
                                    UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                                frame_output.push(IconFrameAction::Face(
                                    face_index,
                                    palette_color_index,
                                    vertex_a,
                                    vertex_b,
                                    vertex_c,
                                    edge_a,
                                    edge_b,
                                    edge_c,
                                ));

                                face_index += 1;
                            }
                        }
                    }

                    output.push(Self::Frame(frame_output));
                }
            }
        }
        Ok(output)
    }
}
