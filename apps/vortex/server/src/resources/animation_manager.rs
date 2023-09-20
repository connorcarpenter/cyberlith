use std::collections::{HashMap, HashSet};

use bevy_ecs::{entity::Entity, system::Resource};

pub struct RotationData {
    frame_entity: Entity
}

impl RotationData {
    fn new(frame_entity: Entity) -> Self {
        Self {
            frame_entity
        }
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

    fn add_rotation(&mut self, entity: Entity) {
        self.rotations.insert(entity);
    }

    fn remove_rotation(&mut self, entity: &Entity) {
        self.rotations.remove(entity);
    }
}

#[derive(Resource)]
pub struct AnimationManager {
    // frame entity -> frame data
    frames: HashMap<Entity, FrameData>,
    // rotation entity -> rotation data
    rotations: HashMap<Entity, RotationData>,
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self {
            rotations: HashMap::new(),
            frames: HashMap::new(),
        }
    }
}

impl AnimationManager {

    pub fn has_frame(&self, entity: &Entity) -> bool {
        self.frames.contains_key(entity)
    }

    pub fn has_rotation(&self, entity: &Entity) -> bool {
        self.rotations.contains_key(entity)
    }

    pub fn on_create_frame(
        &mut self,
        frame_entity: Entity,
    ) {
        self.frames.insert(
            frame_entity,
            FrameData::new(),
        );
    }

    pub fn on_create_rotation(
        &mut self,
        frame_entity: Entity,
        rot_entity: Entity,
    ) {
        let Some(frame_data) = self.frames.get_mut(&frame_entity) else {
            panic!("frame entity not found");
        };
        frame_data.add_rotation(rot_entity);

        self.rotations.insert(
            rot_entity,
            RotationData::new(frame_entity),
        );
    }

    pub fn deregister_frame(&mut self, entity: &Entity) -> Option<FrameData> {
        self.frames.remove(entity)
    }

    pub fn deregister_rotation(&mut self, entity: &Entity) -> RotationData {
        let Some(rot_data) = self.rotations.remove(entity) else {
            panic!("rotation entity not found");
        };

        let frame_entity = rot_data.frame_entity;
        let Some(frame_data) = self.frames.get_mut(&frame_entity) else {
            panic!("frame entity not found");
        };
        frame_data.remove_rotation(entity);

        rot_data
    }

    pub fn on_client_despawn_frame(
        &mut self,
        entity: &Entity,
    ) {
        self.deregister_frame(entity);
    }

    pub fn on_client_despawn_rotation(
        &mut self,
        entity: &Entity,
    ) {
        self.deregister_rotation(entity);
    }
}