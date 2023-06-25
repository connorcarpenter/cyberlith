use std::collections::BTreeMap;

use bevy_ecs::prelude::{Entity, Resource};

use render_api::components::RenderLayer;
use vortex_proto::resources::FileEntryKey;

#[derive(Resource)]
pub struct Global {
    pub project_root_entity: Entity,
    pub changelist: BTreeMap<FileEntryKey, Entity>,
    pub camera_2d: Option<Entity>,
    pub camera_3d: Option<Entity>,
    pub layer_2d: RenderLayer,
    pub layer_3d: RenderLayer,
}

impl Global {
    pub fn new(project_root_entity: Entity) -> Self {
        Self {
            project_root_entity,
            changelist: BTreeMap::new(),
            camera_2d: None,
            camera_3d: None,
            layer_2d: RenderLayer::default(),
            layer_3d: RenderLayer::default(),
        }
    }
}
