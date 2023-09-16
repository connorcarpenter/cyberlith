use bevy_ecs::{entity::Entity, prelude::Commands, system::Resource};

use naia_bevy_client::Client;

use math::Vec2;

#[derive(Resource)]
pub struct AnimationManager {
    pub current_skel_file: Option<Entity>,
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self {
            current_skel_file: None,
        }
    }
}

impl AnimationManager {
    pub(crate) fn drag_edge(
        &mut self,
        _commands: &mut Commands,
        _client: &Client,
        _edge_3d_entity: Entity,
        _mouse_position: Vec2,
        _delta: Vec2,
    ) {
    }

    pub(crate) fn drag_vertex(
        &mut self,
        _commands: &mut Commands,
        _client: &Client,
        _vert_3d_entity: Entity,
        _mouse_position: Vec2,
        _delta: Vec2,
    ) {
    }
}
