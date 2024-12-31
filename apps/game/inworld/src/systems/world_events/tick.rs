use bevy_ecs::{
    change_detection::ResMut, entity::Entity, event::EventReader, prelude::Query,
    system::{Res, SystemState},
};

use game_engine::asset::{AnimatedModelData, AssetHandle, AssetManager, UnitData};

use game_app_network::{
    naia::Tick,
    world::{
        behavior as shared_behavior,
        channels::PlayerCommandChannel,
        components::{NetworkedLastCommand, PhysicsController, TileMovementType},
        messages::PlayerCommands,
        WorldClient, WorldClientTickEvent, WorldServerTickEvent,
    },
};

use crate::{
    components::{
        AnimationState, ClientTileMovement, ConfirmedTileMovement,
        PredictedTileMovement, RenderPosition, TickSkipper,
    },
    resources::{PredictedWorld, Global, InputManager, TickTracker},
};

pub fn client_tick_events(
    mut client: WorldClient,
    global: Res<Global>,
    asset_manager: Res<AssetManager>,
    mut input_manager: ResMut<InputManager>,
    mut predicted_world: ResMut<PredictedWorld>,
    mut tick_reader: EventReader<WorldClientTickEvent>,
    unit_q: Query<(Entity, &AssetHandle<UnitData>, &NetworkedLastCommand)>,
) {
    let owned_entity_opt = global.owned_entity;

    let mut client_ticks = Vec::new();
    for event in tick_reader.read() {
        let client_tick = event.tick;
        client_ticks.push(client_tick);
    }
    if client_ticks.is_empty() {
        return;
    }

    let mut predicted_system_state: SystemState<
        Query<(
            &mut PredictedTileMovement,
            &mut PhysicsController,
            &mut RenderPosition,
            &mut AnimationState,
        )>> = SystemState::new(predicted_world.world_mut());
    let mut character_q = predicted_system_state.get_mut(predicted_world.world_mut());

    for client_tick in client_ticks {

        // process commands
        if let Some(outgoing_command) = input_manager.pop_outgoing_command() {
            // outgoing_command.log(client_tick);

            // send command
            // let distance = client_tile_movement.get_dis();
            // info!("Send Command. Tick: {:?}. MoveDir: {:?}, Dis: {:?}", client_tick, outgoing_command.get_move(), distance);
            client.send_tick_buffer_message::<PlayerCommandChannel, PlayerCommands>(
                &client_tick,
                &outgoing_command,
            );

            input_manager.save_to_command_history(client_tick, Some(outgoing_command.clone()));
            input_manager.recv_incoming_command(client_tick, Some(outgoing_command));
        } else {
            input_manager.save_to_command_history(client_tick, None);
            input_manager.recv_incoming_command(client_tick, None);
        }

        // process tick
        let player_command = input_manager.pop_incoming_command(client_tick);

        for (entity, unit_handle, last_command) in unit_q.iter() {
            let Ok((
                       mut predicted_tile_movement,
                       mut predicted_physics,
                       mut predicted_render_position,
                       mut predicted_animation_state,
                   )) = character_q.get_mut(entity) else {
                continue;
            };
            let Some(animated_model_handle) = asset_manager.get_unit_animated_model_handle(unit_handle) else {
                continue;
            };

            let next_command = {
                if let Some(owned_entity) = owned_entity_opt {
                    if entity == owned_entity {
                        player_command.clone()
                    } else {
                        last_command.to_player_commands()
                    }
                } else {
                    last_command.to_player_commands()
                }
            };
            let client_tile_movement_2: &mut PredictedTileMovement = &mut predicted_tile_movement;
            process_tick(
                &asset_manager,
                TileMovementType::ClientPredicted,
                client_tick,
                next_command,
                client_tile_movement_2,
                &mut predicted_physics,
                &mut predicted_render_position,
                &mut predicted_animation_state,
                animated_model_handle,
            );
        }
    }
}

pub fn process_tick(
    asset_manager: &AssetManager,
    tile_movement_type: TileMovementType,
    tick: Tick,
    player_command: Option<PlayerCommands>,
    tile_movement: &mut dyn ClientTileMovement,
    physics: &mut PhysicsController,
    render_position: &mut RenderPosition,
    animation_state: &mut AnimationState,
    animated_model_handle: &AssetHandle<AnimatedModelData>,
) {
    let lookdir_opt = if let Some(player_command) = player_command.as_ref() {
        player_command.get_look()
    } else {
        None
    };

    let (inner_tile_movement, inner_move_buffer) = tile_movement.decompose();
    shared_behavior::process_tick(
        tile_movement_type,
        tick,
        player_command,
        inner_tile_movement,
        physics,
        inner_move_buffer,
        None,
        None,
    );

    render_position.recv_position(physics.position(), tick);
    animation_state.update(
        &asset_manager,
        animated_model_handle,
        physics.position(),
        physics.velocity(),
        10.0,
    );

    if let Some(lookdir) = lookdir_opt {
        animation_state.recv_lookdir_update(&lookdir);
    }
}

pub fn server_tick_events(
    asset_manager: Res<AssetManager>,
    mut tick_tracker: ResMut<TickTracker>,
    mut tick_reader: EventReader<WorldServerTickEvent>,
    mut unit_q: Query<
        (
            Entity,
            &mut TickSkipper,
            &mut ConfirmedTileMovement,
            &mut PhysicsController,
            &mut RenderPosition,
            &mut AnimationState,
            &AssetHandle<UnitData>,
        ),
    >,
) {
    // TODO here! for components which have received an update for this tick, skip processing!
    for event in tick_reader.read() {
        let server_tick = event.tick;

        // process movement
        for (
            confirmed_entity,
            mut confirmed_tick_skipper,
            mut confirmed_tile_movement,
            mut confirmed_physics,
            mut confirmed_render_position,
            mut confirmed_animation_state,
            unit_handle,
        ) in unit_q.iter_mut()
        {
            if confirmed_tick_skipper.use_skipped_tick(server_tick) {
                panic!(
                    "entity: {:?}, skipping tick: {:?}",
                    confirmed_entity, server_tick
                );
                // continue; // uncomment this to skip processing
            } else {
                // info!("entity: {:?}, processing tick: {:?}", confirmed_entity, server_tick);
            }

            let Some(animated_model_handle) = asset_manager.get_unit_animated_model_handle(unit_handle) else {
                continue;
            };

            let confirmed_tile_movement_2: &mut ConfirmedTileMovement =
                &mut confirmed_tile_movement;
            process_tick(
                &asset_manager,
                TileMovementType::ClientConfirmed,
                server_tick,
                None,
                confirmed_tile_movement_2,
                &mut confirmed_physics,
                &mut confirmed_render_position,
                &mut confirmed_animation_state,
                &animated_model_handle,
            );
        }

        // record
        tick_tracker.set_last_processed_server_tick(server_tick);
    }
}
