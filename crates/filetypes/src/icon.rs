
use naia_serde::{BitReader, FileBitWriter, SerdeInternal as Serde, SerdeErr, UnsignedVariableInteger};

use crate::common::VertexSerdeInt;

#[derive(Debug, Clone)]
enum IconFrameAction {
    //////// x, y//
    Vertex(i16, i16),
    //// vertex id1, vertex id2 //
    Edge(u16, u16),
    //// order_index, palette color index, id1, id2, id3 // (vertex ids) // id4, id5, id6 (edge ids) // TODO: remove order_index?
    Face(u16, u8, u16, u16, u16, u16, u16, u16),
}

#[derive(Serde, Clone, PartialEq)]
enum IconFrameActionType {
    None,
    Vertex,
    Edge,
    Face,
}

// Actions
#[derive(Debug, Clone)]
enum IconAction {
    // path, file_name
    PaletteFile(String, String),
    // frame
    Frame(Vec<IconFrameAction>),
}

#[derive(Serde, Clone, PartialEq)]
enum IconActionType {
    None,
    PaletteFile,
    Frame,
}

// Writer
pub struct IconWriter;

impl IconWriter {

    fn write_from_actions(&self, actions: Vec<IconAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions.iter() {
            match action {
                IconAction::PaletteFile(path, file_name) => {
                    IconActionType::PaletteFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
                IconAction::Frame(frame_actions) => {

                    let mut test_face_index = 0;

                    IconActionType::Frame.ser(&mut bit_writer);

                    for frame_action in frame_actions.iter() {

                        match frame_action {
                            IconFrameAction::Vertex(x, y) => {
                                // continue bit
                                IconFrameActionType::Vertex.ser(&mut bit_writer);

                                // encode X, Y
                                VertexSerdeInt::from(*x).ser(&mut bit_writer);
                                VertexSerdeInt::from(*y).ser(&mut bit_writer);
                            }
                            IconFrameAction::Edge(vertex_a, vertex_b) => {
                                // continue bit
                                IconFrameActionType::Edge.ser(&mut bit_writer);

                                UnsignedVariableInteger::<6>::from(*vertex_a).ser(&mut bit_writer);
                                UnsignedVariableInteger::<6>::from(*vertex_b).ser(&mut bit_writer);
                            }
                            IconFrameAction::Face(
                                face_index,
                                palette_color_index,
                                vertex_a,
                                vertex_b,
                                vertex_c,
                                edge_a,
                                edge_b,
                                edge_c,
                            ) => {
                                if *face_index != test_face_index {
                                    panic!(
                                        "face_index {:?} does not match test_face_index {:?}",
                                        face_index, test_face_index
                                    );
                                }

                                // continue bit
                                IconFrameActionType::Face.ser(&mut bit_writer);

                                palette_color_index.ser(&mut bit_writer);

                                UnsignedVariableInteger::<6>::from(*vertex_a).ser(&mut bit_writer);
                                UnsignedVariableInteger::<6>::from(*vertex_b).ser(&mut bit_writer);
                                UnsignedVariableInteger::<6>::from(*vertex_c).ser(&mut bit_writer);

                                UnsignedVariableInteger::<6>::from(*edge_a).ser(&mut bit_writer);
                                UnsignedVariableInteger::<6>::from(*edge_b).ser(&mut bit_writer);
                                UnsignedVariableInteger::<6>::from(*edge_c).ser(&mut bit_writer);

                                test_face_index += 1;
                            }
                        }
                    }

                    IconFrameActionType::None.ser(&mut bit_writer);
                }

            }
        }

        // continue bit
        IconActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

// Reader
pub struct IconReader;

impl IconReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<IconAction>, SerdeErr> {
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
                    output.push(IconAction::PaletteFile(path, file_name));
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
                                let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                                frame_output.push(IconFrameAction::Edge(vertex_a, vertex_b));
                            }
                            IconFrameActionType::Face => {
                                let palette_color_index = u8::de(bit_reader)?;

                                let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let vertex_c: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                                let edge_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let edge_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                                let edge_c: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                                frame_output.push(IconFrameAction::Face(
                                    face_index, palette_color_index, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c,
                                ));

                                face_index += 1;
                            }
                        }
                    }

                    output.push(IconAction::Frame(frame_output));
                }
            }
        }
        Ok(output)
    }
}