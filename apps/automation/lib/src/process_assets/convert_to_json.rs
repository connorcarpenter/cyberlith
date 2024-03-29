// // use std::collections::HashMap;
//
// use asset_serde::{
//     json::{AssetData, PaletteFile},
//     bits::{AnimAction, FileTransformEntityType, IconAction, IconFrameAction, MeshAction, ModelAction, PaletteAction, SceneAction, SkelAction, SkinAction}
// };
// use asset_serde::json::{AnimFile, AnimFileFrame, IconFile, IconFileFrame, MeshData, ModelFile, SceneFile, SkelFile, SkinFile};
//
// // use crate::process_assets::json::ProcessData;
//
// // pub(crate) fn convert_to_asset_ids(data: &mut AssetData, asset_map: &HashMap<String, ProcessData>) {
// //     match data {
// //         AssetData::Animation(inner) => {
// //             inner.set_skeleton_asset_id = asset_map.get(&inner.skeleton_asset_id).unwrap().asset_id.as_string();
// //         }
// //         AssetData::Icon(inner) => {
// //             inner.palette_asset_id = asset_map.get(&inner.palette_asset_id).unwrap().asset_id.as_string();
// //         }
// //         AssetData::Palette(_) => {
// //             // Do nothing
// //         }
// //         AssetData::Skeleton(_) => {
// //             // Do nothing
// //         }
// //         AssetData::Mesh(_) => {
// //             // Do nothing
// //         }
// //         AssetData::Skin(inner) => {
// //             inner.palette_asset_id = asset_map.get(&inner.palette_asset_id).unwrap().asset_id.as_string();
// //             inner.mesh_asset_id = asset_map.get(&inner.mesh_asset_id).unwrap().asset_id.as_string();
// //         }
// //         AssetData::Scene(inner) => {
// //             for component in &mut inner.components {
// //                 component.asset_id = asset_map.get(&component.asset_id).unwrap().asset_id.as_string();
// //             }
// //         }
// //         AssetData::Model(inner) => {
// //             inner.skeleton_id = asset_map.get(&inner.skeleton_id).unwrap().asset_id.as_string();
// //             for component in &mut inner.components {
// //                 component.asset_id = asset_map.get(&component.asset_id).unwrap().asset_id.as_string();
// //             }
// //         }
// //     }
// // }
//
// pub fn palette(in_bytes: &Vec<u8>) -> AssetData {
//     let actions = PaletteAction::read(in_bytes).unwrap();
//
//     let mut file = PaletteFile::new();
//
//     for action in actions {
//         match action {
//             PaletteAction::Color(r, g, b) => {
//                 file.add_color(r, g, b);
//             }
//         }
//     }
//
//     AssetData::Palette(file)
// }
//
// pub fn skel(in_bytes: &Vec<u8>) -> AssetData {
//     let actions = SkelAction::read(in_bytes).unwrap();
//
//     let mut file = SkelFile::new();
//
//     for action in actions {
//         match action {
//             SkelAction::Vertex(x, y, z, parent_id_opt, vertex_name_opt) => {
//                 file.add_vertex(x, y, z, parent_id_opt.map(|(id, rotation)| (id, rotation.get_inner_value())), vertex_name_opt);
//             }
//         }
//     }
//
//     AssetData::Skeleton(file)
// }
//
// pub fn mesh(in_bytes: &Vec<u8>) -> AssetData {
//     let actions = MeshAction::read(in_bytes).unwrap();
//
//     let mut file = MeshData::new();
//
//     for action in actions {
//         match action {
//             MeshAction::Vertex(x, y, z) => {
//                 file.add_vertex(x, y, z);
//             }
//             MeshAction::Edge(vertex_a, vertex_b) => {
//                 file.add_edge(vertex_a, vertex_b);
//             }
//             MeshAction::Face(face_id, vertex_a, vertex_b, vertex_c, edge_a, edge_b, edge_c) => {
//                 file.add_face(
//                     face_id,
//                     vertex_a,
//                     vertex_b,
//                     vertex_c,
//                     edge_a,
//                     edge_b,
//                     edge_c,
//                 );
//             }
//         }
//     }
//
//     AssetData::Mesh(file)
// }
//
// pub fn anim(in_bytes: &Vec<u8>) -> AssetData {
//     let actions = AnimAction::read(in_bytes).unwrap();
//
//     let mut file = AnimFile::new();
//
//     for action in actions {
//         match action {
//             AnimAction::SkelFile(path, file_name) => {
//                 file.set_skeleton_asset_id(format!("{}/{}", path, file_name).as_str());
//             }
//             AnimAction::ShapeIndex(shape_name) => {
//                 file.add_edge_name(&shape_name);
//             }
//             AnimAction::Frame(poses, transition) => {
//                 let mut frame = AnimFileFrame::new(transition.get_duration_ms());
//
//                 for (shape_index, rotation) in poses {
//                     frame.add_pose(shape_index, rotation.x, rotation.y, rotation.z, rotation.w);
//                 }
//
//                 file.add_frame(frame);
//             }
//         }
//     }
//
//     AssetData::Animation(file)
// }
//
// pub fn icon(in_bytes: &Vec<u8>) -> AssetData {
//     let actions = IconAction::read(in_bytes).unwrap();
//
//     let mut file = IconFile::new();
//
//     for action in actions {
//         match action {
//             IconAction::PaletteFile(path, file_name) => {
//                 file.set_palette_asset_id(format!("{}/{}", path, file_name).as_str());
//             }
//             IconAction::Frame(frame_actions) => {
//                 let mut new_frame = IconFileFrame::new();
//
//                 for frame_action in frame_actions {
//                     match frame_action {
//                         IconFrameAction::Vertex(x, y) => {
//                             new_frame.add_vertex(x, y);
//                         }
//                         IconFrameAction::Edge(start, end) => {
//                             new_frame.add_edge(start, end);
//                         }
//                         IconFrameAction::Face(
//                             face_index,
//                             palette_color_index,
//                             vertex_a_index,
//                             vertex_b_index,
//                             vertex_c_index,
//                             edge_a_index,
//                             edge_b_index,
//                             edge_c_index
//                         ) => {
//                             new_frame.add_face(
//                                 face_index,
//                                 palette_color_index,
//                                 vertex_a_index,
//                                 vertex_b_index,
//                                 vertex_c_index,
//                                 edge_a_index,
//                                 edge_b_index,
//                                 edge_c_index
//                             );
//                         }
//                     }
//                 }
//
//                 file.add_frame(new_frame);
//             }
//         }
//     }
//
//     AssetData::Icon(file)
// }
//
// pub fn skin(in_bytes: &Vec<u8>) -> AssetData {
//     let actions = SkinAction::read(in_bytes).unwrap();
//
//     let mut file = SkinFile::new();
//
//     for action in actions {
//         match action {
//             SkinAction::PaletteFile(path, file_name) => {
//                 file.set_palette_asset_id(format!("{}/{}", path, file_name).as_str());
//             }
//             SkinAction::MeshData(path, file_name) => {
//                 file.set_mesh_asset_id(format!("{}/{}", path, file_name).as_str());
//             }
//             SkinAction::BackgroundColor(palette_color_id) => {
//                 file.set_background_color_id(palette_color_id);
//             }
//             SkinAction::SkinColor(face_id, color_id) => {
//                 file.add_face_color(face_id, color_id);
//             }
//         }
//     }
//
//     AssetData::Skin(file)
// }
//
// pub fn scene(in_bytes: &Vec<u8>) -> AssetData {
//     let actions = SceneAction::read(in_bytes).unwrap();
//
//     let mut file = SceneFile::new();
//
//     for action in actions {
//         match action {
//             SceneAction::SkinOrSceneFile(path, file_name, file_type) => {
//                 file.add_component(format!("{}/{}", path, file_name).as_str(),
//                     match file_type {
//                         FileTransformEntityType::Skin => "skin",
//                         FileTransformEntityType::Scene => "scene",
//                     }
//                 );
//             }
//             SceneAction::NetTransform(
//                 component_id,
//                 x,
//                 y,
//                 z,
//                 scale_x,
//                 scale_y,
//                 scale_z,
//                 rotation
//             ) => {
//                 file.add_transform(
//                     component_id,
//                     x,
//                     y,
//                     z,
//                     scale_x,
//                     scale_y,
//                     scale_z,
//                     rotation.x,
//                     rotation.y,
//                     rotation.z,
//                     rotation.w
//                 );
//             }
//         }
//     }
//
//     AssetData::Scene(file)
// }
//
// pub fn model(in_bytes: &Vec<u8>) -> AssetData {
//     let actions = ModelAction::read(in_bytes).unwrap();
//
//     let mut file = ModelFile::new();
//
//     for action in actions {
//         match action {
//             ModelAction::SkelFile(path, file_name) => {
//                 file.set_skeleton_id(format!("{}/{}", path, file_name).as_str());
//             }
//             ModelAction::SkinOrSceneFile(path, file_name, file_type) => {
//                 file.add_component(
//                     format!("{}/{}", path, file_name).as_str(),
//                     match file_type {
//                         FileTransformEntityType::Skin => "skin",
//                         FileTransformEntityType::Scene => "scene",
//                     }
//                 );
//             }
//             ModelAction::NetTransform(
//                 component_id,
//                 vertex_name,
//                 translation_x,
//                 translation_y,
//                 translation_z,
//                 scale_x,
//                 scale_y,
//                 scale_z,
//                 rotation
//             ) => {
//                 file.add_transform(
//                     component_id,
//                     &vertex_name,
//                     translation_x,
//                     translation_y,
//                     translation_z,
//                     scale_x,
//                     scale_y,
//                     scale_z,
//                     rotation.x,
//                     rotation.y,
//                     rotation.z,
//                     rotation.w,
//                 );
//             }
//         }
//     }
//
//     AssetData::Model(file)
// }