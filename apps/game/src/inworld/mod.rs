
mod systems;
mod resources;

use bevy_app::{App, Plugin, Update};
use bevy_ecs::{prelude::in_state, prelude::not, schedule::IntoSystemConfigs};

use crate::states::AppState;

pub struct InWorldPlugin;

impl Plugin for InWorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, systems::walker_scene::step.run_if(in_state(AppState::InGame)))
            .add_systems(Update, systems::draw::draw.run_if(in_state(AppState::InGame)))
            .add_systems(Update, systems::world::world_connect_events.run_if(not(in_state(AppState::InGame))))
            .add_systems(Update, systems::world::world_spawn_entity_events.run_if(in_state(AppState::InGame)))
            .add_systems(Update, systems::world::world_main_insert_position_events.run_if(in_state(AppState::InGame)))
            .add_systems(Update, systems::world::world_main_insert_asset_ref_events.run_if(in_state(AppState::InGame)))
            .add_systems(Update, systems::world::world_alt1_insert_asset_ref_events.run_if(in_state(AppState::InGame)));
    }
}