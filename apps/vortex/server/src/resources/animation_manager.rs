use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    prelude::Query,
    system::{Commands, Resource},
};

use naia_bevy_server::{CommandsExt, Server};

use vortex_proto::components::AnimFrame;

pub struct RotationData {
    frame_entity: Entity,
}

impl RotationData {
    fn new(frame_entity: Entity) -> Self {
        Self { frame_entity }
    }
}

pub struct FileFrameData {
    // frame entity -> frame data
    frames: HashMap<Entity, FrameData>,
    frame_list: Vec<Option<Entity>>,
}

impl FileFrameData {
    fn new() -> Self {
        Self {
            frames: HashMap::new(),
            frame_list: Vec::new(),
        }
    }

    fn add_frame(
        &mut self,
        frame_entity: Entity,
        frame_order: usize,
        mut frame_q_opt: Option<&mut Query<&mut AnimFrame>>,
    ) {
        self.frames.insert(frame_entity, FrameData::new());

        // add to frame_list
        if frame_order >= self.frame_list.len() {
            self.frame_list.resize(frame_order + 1, None);
            // set frame entity
            self.frame_list[frame_order] = Some(frame_entity);
        } else {
            // move all elements after frame_order up one
            for i in frame_order..self.frame_list.len() {
                // update frame_order in AnimFrame using frame_q_opt
                if let Some(frame_q) = frame_q_opt.as_mut() {
                    if let Ok(mut frame) = frame_q.get_mut(self.frame_list[i].unwrap()) {
                        frame.set_order((i + 1) as u8);
                    }
                }
            }
            self.frame_list.insert(frame_order, Some(frame_entity));
        }

        // for i in 0..self.frame_list.len() {
        //     info!("index: {}, entity: {:?}", i, self.frame_list[i]);
        // }
    }

    fn remove_frame(
        &mut self,
        frame_entity: &Entity,
        frame_q_opt: Option<&mut Query<&mut AnimFrame>>,
    ) -> Option<FrameData> {
        let Some(frame_data) = self.frames.remove(frame_entity) else {
            panic!("frame data not found");
        };

        let frame_order = {
            let mut frame_order_opt = None;
            for (frame_index, frame_item) in self.frame_list.iter().enumerate() {
                if let Some(frame_item) = frame_item {
                    if frame_item == frame_entity {
                        frame_order_opt = Some(frame_index);
                        break;
                    }
                }
            }
            frame_order_opt.unwrap()
        };

        // get frame_order of frame_entity
        if let Some(frame_q) = frame_q_opt {
            // move all elements after frame_order down one
            for i in frame_order..self.frame_list.len() - 1 {
                self.frame_list[i] = self.frame_list[i + 1];

                // update frame_order in AnimFrame using frame_q_opt
                if let Ok(mut frame) = frame_q.get_mut(self.frame_list[i].unwrap()) {
                    frame.set_order(i as u8);
                }
            }

            self.frame_list.truncate(self.frame_list.len() - 1);
        }

        Some(frame_data)
    }

    fn add_rotation(&mut self, frame_entity: Entity, rotation_entity: Entity) {
        let Some(frame_data) = self.frames.get_mut(&frame_entity) else {
            panic!("frame entity not found");
        };
        frame_data.add_rotation(rotation_entity);
    }

    fn remove_rotation(&mut self, frame_entity: &Entity, rotation_entity: &Entity) {
        let Some(frame_data) = self.frames.get_mut(&frame_entity) else {
            panic!("frame entity not found");
        };
        frame_data.remove_rotation(rotation_entity);
    }
}

pub struct FrameData {
    rotations: HashSet<Entity>,
}

impl FrameData {
    fn new() -> Self {
        Self {
            rotations: HashSet::new(),
        }
    }

    fn add_rotation(&mut self, rotation_entity: Entity) {
        self.rotations.insert(rotation_entity);
    }

    fn remove_rotation(&mut self, rotation_entity: &Entity) {
        self.rotations.remove(rotation_entity);
    }
}

#[derive(Resource)]
pub struct AnimationManager {
    // file entity -> file frame data
    file_frame_data: HashMap<Entity, FileFrameData>,
    // rotation entity -> rotation data
    rotations: HashMap<Entity, RotationData>,
    // frame_entity -> file_entity
    frames: HashMap<Entity, Entity>,
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self {
            rotations: HashMap::new(),
            file_frame_data: HashMap::new(),
            frames: HashMap::new(),
        }
    }
}

impl AnimationManager {
    pub fn has_frame(&self, frame_entity: &Entity) -> bool {
        self.frames.contains_key(frame_entity)
    }

    pub fn has_rotation(&self, rotation_entity: &Entity) -> bool {
        self.rotations.contains_key(rotation_entity)
    }

    pub fn on_create_frame(
        &mut self,
        file_entity: &Entity,
        frame_entity: &Entity,
        frame_index: usize,
        frame_q_opt: Option<&mut Query<&mut AnimFrame>>,
    ) {
        if !self.file_frame_data.contains_key(file_entity) {
            self.file_frame_data
                .insert(*file_entity, FileFrameData::new());
        }
        let file_frame_data = self.file_frame_data.get_mut(file_entity).unwrap();
        file_frame_data.add_frame(*frame_entity, frame_index, frame_q_opt);

        self.frames.insert(*frame_entity, *file_entity);
    }

    pub fn on_create_rotation(&mut self, frame_entity: Entity, rot_entity: Entity) {
        let Some(file_entity) = self.frames.get(&frame_entity) else {
            panic!("frame entity not found");
        };

        let Some(file_frame_data) = self.file_frame_data.get_mut(&file_entity) else {
            panic!("frame entity not found for file");
        };
        file_frame_data.add_rotation(frame_entity, rot_entity);

        self.rotations
            .insert(rot_entity, RotationData::new(frame_entity));
    }

    pub fn on_despawn_frame(
        &mut self,
        commands: &mut Commands,
        server: &mut Server,
        frame_entity: &Entity,
        frame_q_opt: Option<&mut Query<&mut AnimFrame>>,
    ) {
        let frame_data = self.deregister_frame(frame_entity, frame_q_opt).unwrap();
        for rotation_entity in frame_data.rotations {
            commands
                .entity(rotation_entity)
                .take_authority(server)
                .despawn();
            self.deregister_rotation(&rotation_entity);
        }
    }

    pub fn on_despawn_rotation(&mut self, rotation_entity: &Entity) {
        self.deregister_rotation(rotation_entity);
    }

    pub fn deregister_frame(
        &mut self,
        frame_entity: &Entity,
        frame_q_opt: Option<&mut Query<&mut AnimFrame>>,
    ) -> Option<FrameData> {
        let Some(file_entity) = self.frames.remove(frame_entity) else {
            panic!("frame entity not found");
        };

        let Some(file_frame_data) = self.file_frame_data.get_mut(&file_entity) else {
            panic!("frame entity not found for file");
        };
        let output = file_frame_data.remove_frame(frame_entity, frame_q_opt);
        if file_frame_data.frames.is_empty() {
            self.file_frame_data.remove(&file_entity);
        }

        output
    }

    pub fn deregister_rotation(&mut self, rotation_entity: &Entity) -> RotationData {
        let Some(rot_data) = self.rotations.remove(rotation_entity) else {
            panic!("rotation entity not found");
        };

        let frame_entity = rot_data.frame_entity;
        if let Some(file_entity) = self.frames.get(&frame_entity) {
            if let Some(frame_data) = self.file_frame_data.get_mut(&file_entity) {
                frame_data.remove_rotation(&frame_entity, rotation_entity);
            }
        }

        rot_data
    }
}
