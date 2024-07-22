
use bevy_app::{App, Plugin, Update};
use bevy_ecs::{prelude::in_state, prelude::not, schedule::IntoSystemConfigs};

use game_app_common::AppState;

use crate::systems;

pub struct InWorldPlugin;

impl Plugin for InWorldPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                Update,
                (
                    systems::world_events::connect_events,
                )
                    .run_if(not(in_state(AppState::InGame)))
            )
            .add_systems(
                Update,
                (
                    systems::render::draw_models,
                    systems::world_events::spawn_entity_events,
                    systems::world_events::insert_position_events,
                    systems::assets::main_insert_asset_ref_events,
                    systems::assets::alt1_insert_asset_ref_events,
                )
                    .run_if(in_state(AppState::InGame))
            );
    }
}