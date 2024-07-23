
use bevy_app::{App, Plugin, Update};
use bevy_ecs::{prelude::in_state, prelude::not, schedule::{IntoSystemConfigs, IntoSystemSetConfigs}};

use game_app_common::AppState;
use game_engine::naia::ReceiveEvents;

use crate::{systems, resources::Global};

pub struct InWorldPlugin;

impl Plugin for InWorldPlugin {
    fn build(&self, app: &mut App) {
        app
            // resources
            .init_resource::<Global>()
            // systems
            .add_systems(
                Update,
                (
                    systems::world_events::connect_events,
                    systems::world_events::reject_events,
                )
                    .run_if(not(in_state(AppState::InGame)))
                    .in_set(ReceiveEvents)
            )
            .add_systems(
                Update,
                (
                    systems::world_events::disconnect_events,
                    systems::world_events::spawn_entity_events,
                    systems::world_events::despawn_entity_events,
                    systems::world_events::insert_position_events,
                    systems::world_events::update_position_events,
                    systems::world_events::remove_position_events,
                    systems::world_events::message_events,
                    systems::world_events::late_animation_handle_add,
                    systems::world_events::late_model_handle_add,

                    systems::assets::main_insert_asset_ref_events,
                    systems::assets::alt1_insert_asset_ref_events,

                    systems::render::draw_models,
                )
                    .run_if(in_state(AppState::InGame))
                    .in_set(ReceiveEvents)
            )
            // Tick Event
            .configure_sets(Update, systems::Tick.after(ReceiveEvents))
            .add_systems(
                Update,
                (
                    systems::world_events::tick_events,
                )
                    .run_if(in_state(AppState::InGame))
                    .in_set(systems::Tick)
            )
            // Realtime Gameplay Loop
            .configure_sets(Update, systems::MainLoop.after(systems::Tick))
            .add_systems(
                Update,
                (
                    systems::input::key_input,
                    systems::sync::sync_clientside_sprites,
                    systems::sync::sync_serverside_sprites,
                )
                    .run_if(in_state(AppState::InGame))
                    .in_set(systems::MainLoop),
            );
    }
}