use bevy_ecs::{prelude::Commands, entity::Entity, system::Resource};

use naia_bevy_client::Client;
use math::Vec2;

#[derive(Resource)]
pub struct AnimationManager {

}

impl Default for AnimationManager {
    fn default() -> Self {
        Self {

        }
    }
}

impl AnimationManager {
    pub(crate) fn drag_edge(&mut self, commands: &mut Commands, client: &Client, edge_3d_entity: Entity, mouse_position: Vec2, delta: Vec2) {
        todo!()
    }

    pub(crate) fn drag_vertex(&mut self, commands: &mut Commands, client: &Client, vert_3d_entity: Entity, mouse_position: Vec2, delta: Vec2) {
        todo!()
    }
}