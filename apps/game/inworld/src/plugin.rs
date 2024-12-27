use bevy_app::{App, Plugin, Update};
use bevy_ecs::{
    prelude::not,
    schedule::{IntoSystemConfigs, IntoSystemSetConfigs},
};
use bevy_state::condition::in_state;

use game_app_common::AppState;
use game_app_network::naia::ReceiveEvents;

use crate::{
    resources::{Global, InputManager, RollbackManager, TickTracker},
    systems,
    systems::world_events::PredictionEvents,
};

pub struct InWorldPlugin;

impl Plugin for InWorldPlugin {
    fn build(&self, app: &mut App) {
        app
            // resources
            .init_resource::<Global>()
            .init_resource::<InputManager>()
            .init_resource::<PredictionEvents>()
            .init_resource::<TickTracker>()
            .init_resource::<RollbackManager>()
            // systems
            .add_systems(
                Update,
                (
                    systems::world_events::connect_events,
                    systems::world_events::reject_events,
                )
                    .run_if(not(in_state(AppState::InGame)))
                    .in_set(ReceiveEvents),
            )
            .add_systems(
                Update,
                (
                    systems::world_events::disconnect_events,
                    systems::world_events::message_events,
                    systems::world_events::spawn_entity_events,
                    systems::world_events::despawn_entity_events,
                    systems::world_events::insert_next_tile_position_events,
                    systems::world_events::update_next_tile_position_events,
                    systems::world_events::remove_next_tile_position_events,
                    systems::world_events::insert_look_direction_events,
                    systems::world_events::update_look_direction_events,
                    systems::world_events::remove_look_direction_events,
                    systems::world_events::insert_net_move_buffer_events,
                    systems::world_events::update_net_move_buffer_events,
                    systems::world_events::remove_net_move_buffer_events,
                    systems::world_events::insert_asset_ref_events,
                    PredictionEvents::process,
                )
                    .run_if(in_state(AppState::InGame))
                    .in_set(ReceiveEvents),
            )
            // Rollback Event
            .configure_sets(Update, systems::Rollback.after(ReceiveEvents))
            .add_systems(
                Update,
                RollbackManager::execute_rollback
                    .run_if(in_state(AppState::InGame))
                    .in_set(systems::Rollback),
            )
            // Tick Event
            .configure_sets(Update, systems::Tick.after(systems::Rollback))
            .add_systems(
                Update,
                (
                    systems::world_events::client_tick_events,
                    systems::world_events::server_tick_events,
                )
                    .run_if(in_state(AppState::InGame))
                    .in_set(systems::Tick),
            )
            // Realtime Gameplay Loop
            .configure_sets(Update, systems::MainLoop.after(systems::Tick))
            .add_systems(
                Update,
                (InputManager::recv_key_input)
                    .run_if(in_state(AppState::InGame))
                    .in_set(systems::MainLoop),
            )
            // Render
            .configure_sets(Update, systems::Render.after(systems::MainLoop))
            .add_systems(
                Update,
                (systems::render::draw_units,)
                    .run_if(in_state(AppState::InGame))
                    .in_set(systems::Render),
            );
    }
}
