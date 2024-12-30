use bevy_ecs::{system::SystemState, entity::Entity, world::World, prelude::{Resource, Query}};

use game_app_network::world::components::PhysicsController;

use game_engine::logging::info;

use crate::components::{AnimationState, ConfirmedTileMovement, PredictedTileMovement, RenderPosition};

#[derive(Resource)]
pub struct PredictedWorld {
    world: World,
}

impl Default for PredictedWorld {
    fn default() -> Self {
        Self { world: World::default() }
    }
}

impl PredictedWorld {
    pub fn world(&self) -> &World {
        &self.world
    }

    pub fn world_mut(&mut self) -> &mut World {
        &mut self.world
    }

    fn clear(&mut self) {
        self.world.clear_entities();

        // TODO: clear specific resources, making sure to not clear any schedules!
    }

    pub fn extract(&mut self, main: &mut World) {

        info!("PredictedWorld::extract()");

        let mut system_state: SystemState<Query<(
                Entity,
                &ConfirmedTileMovement,
                &PhysicsController,
                &RenderPosition,
                &AnimationState
        )>> = SystemState::new(main);
        let character_q = system_state.get(main);

        let mut new_entities = Vec::new();
        for (
            confirmed_entity,
            confirmed_tile_movement,
            confirmed_physics,
            confirmed_render_pos,
            confirmed_animation_state
        ) in character_q.iter() {

            // Get old predicted render position
            let old_predicted_render_pos_opt = self.world.get::<RenderPosition>(confirmed_entity);

            let predicted_tile_movement = PredictedTileMovement::from(confirmed_tile_movement);
            let predicted_physics = confirmed_physics.clone();
            let predicted_render_pos = RenderPosition::extract(confirmed_render_pos, old_predicted_render_pos_opt);
            let predicted_animation_state = confirmed_animation_state.clone();

            info!("PredictedWorld::extract: confirmed_entity: {:?}", confirmed_entity);

            new_entities.push((confirmed_entity, (predicted_tile_movement, predicted_physics, predicted_render_pos, predicted_animation_state)));
        }

        // clear the world of entities
        self.clear();

        // add new entities to predicted world
        self.world.insert_or_spawn_batch(new_entities).unwrap();
    }

    pub fn tick(&mut self) {

    }
}