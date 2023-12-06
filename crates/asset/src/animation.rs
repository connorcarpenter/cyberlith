use std::collections::HashMap;
use std::fs;

use bevy_log::info;

use naia_serde::BitReader;

use math::Quat;
use render_api::{AssetHash, Handle, components::Transform};

use crate::{asset_dependency::{AssetDependency, SkinOrSceneHandle}, AssetHandle, ModelData, SkeletonData, asset_handle::AssetHandleImpl};

impl AssetHash<AnimationData> for String {}

pub struct AnimationData {
    skeleton_file: AssetDependency<SkeletonData>,
    bone_names: Vec<String>,
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
    rotations: Vec<(usize, Quat)>,
}

impl Frame {
    pub fn new(duration_ms: f32) -> Self {
        Self {
            duration_ms,
            rotations: Vec::new(),
        }
    }

    pub(crate) fn add_rotation(&mut self, bone_index: usize, rotation: Quat) {
        self.rotations.push((bone_index, rotation));
    }
}

impl AnimationData {
    pub(crate) fn load_dependencies(&self, handle: Handle<Self>, dependencies: &mut Vec<(AssetHandle, String)>) {
        let AssetDependency::<SkeletonData>::Path(path) = &self.skeleton_file else {
            panic!("expected path right after load");
        };
        dependencies.push((handle.into(), path.clone()));
    }

    pub(crate) fn finish_dependency(&mut self, _dependency_path: String, dependency_handle: AssetHandle) {
        match dependency_handle.to_impl() {
            AssetHandleImpl::Skeleton(handle) => {
                self.skeleton_file.load_handle(handle);
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }
    }

    pub(crate) fn get_skeleton_handle(&self) -> Handle<SkeletonData> {
        if let AssetDependency::<SkeletonData>::Handle(handle) = &self.skeleton_file {
            *handle
        } else {
            panic!("expected skeleton handle");
        }
    }

    pub fn get_duration(&self) -> f32 {
        self.total_duration
    }

    pub(crate) fn get_animated_components(&self, skeleton_data: &SkeletonData, model_data: &ModelData, frame_elapsed_ms: f32) -> Option<Vec<(SkinOrSceneHandle, Transform)>> {
        let model_components = model_data.get_components_copied();

        let (frame_index, next_frame_index, interpolation) = self.get_frame_stats(frame_elapsed_ms);

        let interpolated_skeleton = self.get_interpolated_skeleton(skeleton_data, frame_index, next_frame_index, interpolation);

        let mut output = Vec::new();
        for (component_handle, bone_name, child_transform) in model_components {
            let parent_transform = interpolated_skeleton.get(&bone_name).expect("bone name not found in skeleton");
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

    fn get_interpolated_skeleton(&self, skeleton_data: &SkeletonData, frame_index: usize, next_frame_index: usize, interpolation: f32) -> HashMap<String, Transform> {


        let current_frame = &self.frames[frame_index];
        let next_frame = &self.frames[next_frame_index];

        let mut interpolated_rotations = HashMap::new();
        let max_rotations = current_frame.rotations.len().max(next_frame.rotations.len());
        for rot_index in 0..max_rotations {

            let mut bone_index = 0;
            let current_rotation = {
                if let Some((bone_index_t, current_rot)) = current_frame.rotations.get(rot_index) {
                    bone_index = *bone_index_t;
                    *current_rot
                } else {
                    Quat::IDENTITY
                }
            };

            let next_rotation = {
                if let Some((bone_index_t, next_rot)) = next_frame.rotations.get(rot_index) {
                    bone_index = *bone_index_t;
                    *next_rot
                } else {
                    Quat::IDENTITY
                }
            };

            let interpolated_rotation = current_rotation.slerp(next_rotation, interpolation);

            let bone_name = &self.bone_names[bone_index];

            interpolated_rotations.insert(bone_name.clone(), interpolated_rotation);
        }

        skeleton_data.get_interpolated_skeleton(interpolated_rotations)
    }
}

impl From<String> for AnimationData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let mut bit_reader = BitReader::new(&data);

        let actions =
            filetypes::AnimAction::read(&mut bit_reader).expect("unable to parse file");

        let mut skel_file_opt = None;
        let mut names = Vec::new();
        let mut frames = Vec::new();
        let mut total_animation_time_ms = 0.0;
        for action in actions {
            match action {
                filetypes::AnimAction::SkelFile(path, file_name) => {
                    info!("SkelFile: {}/{}", path, file_name);
                    skel_file_opt = Some(format!("{}/{}", path, file_name));
                }
                filetypes::AnimAction::ShapeIndex(name) => {
                    info!("ShapeIndex: {}", name);
                    names.push(name);
                }
                filetypes::AnimAction::Frame(rotation_map, transition_time) => {
                    info!("Frame {}: {:?}ms", frames.len(), transition_time.get_duration_ms());
                    let transition_time = transition_time.get_duration_ms() as f32;
                    let mut frame = Frame::new(transition_time);
                    total_animation_time_ms += transition_time;
                    for (name_index, rotation) in rotation_map {
                        info!("name_index: {} . rotation: ({:?}, {:?}, {:?}, {:?})", name_index, rotation.x, rotation.y, rotation.z, rotation.w);
                        frame.add_rotation(name_index as usize, Quat::from_xyzw(rotation.x, rotation.y, rotation.z, rotation.w));

                    }
                    frames.push(frame);
                }
            }
        }

        // todo: lots here

        Self {
            skeleton_file: AssetDependency::Path(skel_file_opt.unwrap()),
            bone_names: names,
            frames,
            total_duration: total_animation_time_ms,
        }
    }
}