use naia_serde::{BitReader, FileBitWriter, SerdeInternal as Serde, SerdeErr, UnsignedVariableInteger};

use crate::common::VertexSerdeInt;

// Actions
#[derive(Debug, Clone)]
enum MeshAction {
    //////// x,   y,   z //
    Vertex(i16, i16, i16),
    //// id1, id2 // (vertex ids)
    Edge(u16, u16),
    //// order_index, id1, id2, id3 // (vertex ids) // id4, id5, id6 (edge ids)
    Face(u16, u16, u16, u16, u16, u16, u16),
}

#[derive(Serde, Clone, PartialEq)]
enum MeshActionType {
    None,
    Vertex,
    Edge,
    Face,
}

// Writer
pub struct MeshWriter;

impl MeshWriter {

    fn write_from_actions(&self, actions: Vec<MeshAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        let mut test_face_index = 0;
        for action in actions.iter() {
            match action {
                MeshAction::Vertex(x, y, z) => {
                    // continue bit
                    MeshActionType::Vertex.ser(&mut bit_writer);

                    // encode X, Y, Z
                    VertexSerdeInt::from(*x).ser(&mut bit_writer);
                    VertexSerdeInt::from(*y).ser(&mut bit_writer);
                    VertexSerdeInt::from(*z).ser(&mut bit_writer);
                }
                MeshAction::Edge(vertex_a, vertex_b) => {
                    // continue bit
                    MeshActionType::Edge.ser(&mut bit_writer);

                    UnsignedVariableInteger::<6>::from(*vertex_a).ser(&mut bit_writer);
                    UnsignedVariableInteger::<6>::from(*vertex_b).ser(&mut bit_writer);
                }
                MeshAction::Face(
                    face_index,
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
                    MeshActionType::Face.ser(&mut bit_writer);

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

        // continue bit
        MeshActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

// Reader
pub struct MeshReader;

impl MeshReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<MeshAction>, SerdeErr> {
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

                    output.push(MeshAction::Vertex(x, y, z));
                }
                MeshActionType::Edge => {
                    let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    output.push(MeshAction::Edge(vertex_a, vertex_b));
                }
                MeshActionType::Face => {
                    let vertex_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let vertex_c: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    let edge_a: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let edge_b: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();
                    let edge_c: u16 = UnsignedVariableInteger::<6>::de(bit_reader)?.to();

                    output.push(MeshAction::Face(
                        face_index, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c,
                    ));

                    face_index += 1;
                }
            }
        }
        Ok(output)
    }
}