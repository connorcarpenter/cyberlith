use naia_serde::{FileBitWriter, SerdeInternal as Serde, UnsignedVariableInteger};

use crate::{
    common::VertexSerdeInt,
    icon::{IconActionType, IconFrameAction, IconFrameActionType},
    IconAction,
};

impl IconAction {
    pub fn write(actions: Vec<Self>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions.iter() {
            match action {
                Self::PaletteFile(path, file_name) => {
                    IconActionType::PaletteFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
                Self::Frame(frame_actions) => {
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
