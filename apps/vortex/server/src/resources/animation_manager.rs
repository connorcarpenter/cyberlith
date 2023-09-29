use std::collections::{HashMap, HashSet};

use bevy_ecs::{entity::Entity, system::Resource};

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
}

impl FileFrameData {
    fn new() -> Self {
        Self {
            frames: HashMap::new(),
        }
    }

    fn add_frame(&mut self, frame_entity: Entity, frame_order: usize) {
        self.frames.insert(frame_entity, FrameData::new(frame_order));
    }

    fn remove_frame(&mut self, frame_entity: &Entity) -> Option<FrameData> {
        self.frames.remove(frame_entity)
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
    order: usize,
    rotations: HashSet<Entity>,
}

impl FrameData {
    fn new(order: usize) -> Self {
        Self {
            order,
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

    pub fn on_create_frame(&mut self, file_entity: &Entity, frame_entity: &Entity, frame_index: usize) {
        if !self.file_frame_data.contains_key(file_entity) {
            self.file_frame_data.insert(*file_entity, FileFrameData::new());
        }
        let file_frame_data = self.file_frame_data.get_mut(file_entity).unwrap();
        file_frame_data.add_frame(*frame_entity, frame_index);

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

    pub fn on_despawn_frame(&mut self, frame_entity: &Entity) {
        self.deregister_frame(frame_entity);
    }

    pub fn on_despawn_rotation(&mut self, rotation_entity: &Entity) {
        self.deregister_rotation(rotation_entity);
    }

    pub fn deregister_frame(&mut self, frame_entity: &Entity) -> Option<FrameData> {
        let Some(file_entity) = self.frames.remove(frame_entity) else {
            panic!("frame entity not found");
        };

        let Some(file_frame_data) = self.file_frame_data.get_mut(&file_entity) else {
            panic!("frame entity not found for file");
        };
        file_frame_data.remove_frame(frame_entity)
    }

    pub fn deregister_rotation(&mut self, rotation_entity: &Entity) -> RotationData {
        let Some(rot_data) = self.rotations.remove(rotation_entity) else {
            panic!("rotation entity not found");
        };

        let frame_entity = rot_data.frame_entity;
        let Some(file_entity) = self.frames.get(&frame_entity) else {
            panic!("frame entity not found");
        };
        if let Some(frame_data) = self.file_frame_data.get_mut(&file_entity) {
            frame_data.remove_rotation(&frame_entity, rotation_entity);
        };

        rot_data
    }
}
