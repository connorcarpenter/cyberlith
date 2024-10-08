use std::collections::HashMap;

use math::Quat;
use render_api::components::Transform;

use crate::{
    asset_dependency::{AssetComponentHandle, AssetDependency},
    AssetHandle, ModelData, SkeletonData, TypedAssetId,
};

pub struct AnimationData {
    skeleton_file: AssetDependency<SkeletonData>,
    frames: Vec<Frame>,
    total_duration: f32,
}

impl Default for AnimationData {
    fn default() -> Self {
        panic!("");
    }
}

struct Frame {
    duration_ms: f32,
    // bone index, rotation
    rotations: HashMap<String, Quat>,
}

impl Frame {
    pub fn new(duration_ms: f32) -> Self {
        Self {
            duration_ms,
            rotations: HashMap::new(),
        }
    }

    pub(crate) fn add_rotation(&mut self, bone_name: String, rotation: Quat) {
        self.rotations.insert(bone_name, rotation);
    }
}

impl AnimationData {
    pub(crate) fn load_dependencies(
        &self,
        asset_handle: AssetHandle<Self>,
        dependencies: &mut Vec<(TypedAssetId, TypedAssetId)>,
    ) {
        let AssetDependency::<SkeletonData>::AssetId(asset_id) = &self.skeleton_file else {
            panic!("expected path right after load");
        };
        dependencies.push((asset_handle.into(), TypedAssetId::Skeleton(*asset_id)));
    }

    pub(crate) fn finish_dependency(&mut self, dependency_typed_id: TypedAssetId) {
        match dependency_typed_id {
            TypedAssetId::Skeleton(id) => {
                let handle = AssetHandle::<SkeletonData>::new(id);
                self.skeleton_file.load_asset_handle(handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }

    pub fn get_skeleton_handle(&self) -> AssetHandle<SkeletonData> {
        if let AssetDependency::<SkeletonData>::AssetHandle(handle) = &self.skeleton_file {
            *handle
        } else {
            panic!("expected skeleton handle");
        }
    }

    pub fn get_duration_ms(&self) -> f32 {
        self.total_duration
    }

    pub fn get_frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn get_animated_components(
        &self,
        skeleton_data: &SkeletonData,
        model_data: &ModelData,
        frame_elapsed_ms: f32,
    ) -> Option<Vec<(AssetComponentHandle, Transform)>> {
        let model_components = model_data.get_components_copied();

        let (frame_index, next_frame_index, interpolation) = self.get_frame_stats(frame_elapsed_ms);
        //info!("frame_index: {}, next_frame_index: {}, interpolation: {}", frame_index, next_frame_index, interpolation);

        let interpolated_skeleton = self.get_interpolated_skeleton(
            skeleton_data,
            frame_index,
            next_frame_index,
            interpolation,
        );

        let mut output = Vec::new();
        for (component_handle, bone_name, child_transform) in model_components {
            let parent_transform = interpolated_skeleton
                .get(&bone_name)
                .expect("bone name not found in skeleton");
            let final_transform = child_transform.multiply(parent_transform);
            output.push((component_handle, final_transform));
        }

        Some(output)
    }

    // returns (frame_index, next_frame_index, interpolation)
    fn get_frame_stats(&self, frame_elapsed_ms: f32) -> (usize, usize, f32) {
        let mut remaining_ms = frame_elapsed_ms;
        let mut frame_index = 0;
        loop {
            let frame_duration = self.frames[frame_index].duration_ms;
            if remaining_ms > frame_duration {
                remaining_ms -= frame_duration;

                if frame_index + 1 >= self.frames.len() {
                    return (frame_index, 0, 1.0);
                } else {
                    frame_index += 1;
                    continue;
                }
            } else {
                let mut next_frame_index = frame_index + 1;
                if next_frame_index >= self.frames.len() {
                    next_frame_index = 0;
                }
                return (frame_index, next_frame_index, remaining_ms / frame_duration);
            }
        }
    }

    fn get_interpolated_skeleton(
        &self,
        skeleton_data: &SkeletonData,
        frame_index: usize,
        next_frame_index: usize,
        interpolation: f32,
    ) -> HashMap<String, Transform> {
        let current_frame = &self.frames[frame_index];
        let next_frame = &self.frames[next_frame_index];

        let mut interpolated_rotations = HashMap::new();

        for (bone_name, current_rotation) in current_frame.rotations.iter() {
            let next_rotation = {
                if let Some(next_rot) = next_frame.rotations.get(bone_name) {
                    *next_rot
                } else {
                    Quat::IDENTITY
                }
            };

            let interpolated_rotation = current_rotation.slerp(next_rotation, interpolation);

            interpolated_rotations.insert(bone_name.clone(), interpolated_rotation);
        }

        for (bone_name, next_rotation) in next_frame.rotations.iter() {
            if !interpolated_rotations.contains_key(bone_name) {
                let current_rotation = {
                    if let Some(current_rot) = current_frame.rotations.get(bone_name) {
                        *current_rot
                    } else {
                        Quat::IDENTITY
                    }
                };

                let interpolated_rotation = current_rotation.slerp(*next_rotation, interpolation);

                interpolated_rotations.insert(bone_name.clone(), interpolated_rotation);
            }
        }

        skeleton_data.get_interpolated_skeleton(interpolated_rotations)
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        let actions = asset_serde::bits::AnimAction::read(bytes).expect("unable to parse file");

        let mut skeleton_asset_id = None;
        let mut name_map = HashMap::new();
        let mut frames = Vec::new();
        let mut total_animation_time_ms = 0.0;
        for action in actions {
            match action {
                asset_serde::bits::AnimAction::SkelFile(asset_id) => {
                    // info!("SkelFile: {}", asset_id.as_string());
                    skeleton_asset_id = Some(asset_id);
                }
                asset_serde::bits::AnimAction::ShapeIndex(name) => {
                    //info!("ShapeIndex {}: {}", names.len(), name);
                    name_map.insert(name_map.len() as u16, name);
                }
                asset_serde::bits::AnimAction::Frame(rotation_map, transition_time) => {
                    // info!(
                    //     "Frame {}: {:?}ms",
                    //     frames.len(),
                    //     transition_time.get_duration_ms()
                    // );
                    let transition_time = transition_time.get_duration_ms() as f32;
                    let mut frame = Frame::new(transition_time);
                    total_animation_time_ms += transition_time;
                    for (name_index, rotation) in rotation_map {
                        let name = name_map.get(&name_index).unwrap().clone();
                        // info!(
                        //     "name: {} . rotation: ({:?}, {:?}, {:?}, {:?})",
                        //     &name, rotation.x, rotation.y, rotation.z, rotation.w
                        // );
                        frame.add_rotation(
                            name,
                            Quat::from_xyzw(rotation.x, rotation.y, rotation.z, rotation.w),
                        );
                    }
                    frames.push(frame);
                }
            }
        }

        Self {
            skeleton_file: AssetDependency::AssetId(skeleton_asset_id.unwrap()),
            frames,
            total_duration: total_animation_time_ms,
        }
    }
}
