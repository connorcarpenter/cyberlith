use std::collections::HashMap;

use bevy_ecs::{
    change_detection::ResMut,
    event::EventReader,
    prelude::{Commands, Query},
    system::Res,
};

use game_engine::{
    asset::{AssetHandle, AssetManager, UnitData},
    logging::info,
    math::{Quat, Vec3},
    render::components::{RenderLayers, Transform, Visibility},
};

use game_app_network::{
    naia::sequence_greater_than,
    world::{
        // behavior as shared_behavior,
        components::{NetworkedTileTarget, PhysicsController},
        WorldClient,
        WorldInsertComponentEvent,
        WorldRemoveComponentEvent,
        WorldUpdateComponentEvent,
    },
};

use crate::{
    components::{AnimationState, ConfirmedTileMovement, RenderPosition, TickSkipper},
    resources::{RollbackManager, TickTracker},
};

pub fn insert_net_tile_target_events(
    client: WorldClient,
    mut commands: Commands,
    net_tile_target_q: Query<&NetworkedTileTarget>,
    mut rollback_manager: ResMut<RollbackManager>,
    mut event_reader: EventReader<WorldInsertComponentEvent<NetworkedTileTarget>>,
) {
    let Some(server_tick) = client.server_tick() else {
        return;
    };

    for event in event_reader.read() {
        let server_tick_instant = client
            .tick_to_instant(server_tick)
            .expect("failed to convert tick to instant");
        let entity = event.entity;

        info!(
            "received Inserted Component: `NetworkedTileTarget` from World Server! (entity: {:?})",
            entity
        );

        let net_tile_target = net_tile_target_q.get(entity).unwrap();

        rollback_manager.add_event(server_tick);

        let layer = RenderLayers::layer(0);

        commands
            .entity(entity)
            // Insert Position stuff
            .insert(ConfirmedTileMovement::new_stopped(net_tile_target))
            .insert(PhysicsController::new(net_tile_target))
            .insert(TickSkipper::new())
            // Insert other Rendering Stuff
            .insert(AnimationState::new())
            .insert(RenderPosition::new(
                net_tile_target,
                server_tick,
                server_tick_instant,
            ))
            .insert(layer)
            .insert(Visibility::default())
            .insert(
                Transform::from_translation(Vec3::splat(0.0))
                    .with_rotation(Quat::from_rotation_z(f32::to_radians(90.0))),
            );
    }
}

pub fn update_net_tile_target_events(
    asset_manager: Res<AssetManager>,
    tick_tracker: Res<TickTracker>,
    mut rollback_manager: ResMut<RollbackManager>,
    mut event_reader: EventReader<WorldUpdateComponentEvent<NetworkedTileTarget>>,
    mut updated_q: Query<(
        &NetworkedTileTarget,
        &mut ConfirmedTileMovement,
        &mut PhysicsController,
        &mut RenderPosition,
        &mut AnimationState,
        &AssetHandle<UnitData>,
    )>,
    mut tick_skipper_q: Query<&mut TickSkipper>,
) {
    let mut events = HashMap::new();
    for event in event_reader.read() {
        let server_tick = event.tick;
        let updated_entity = event.entity;

        if !events.contains_key(&updated_entity) {
            events.insert(updated_entity, server_tick);
        } else {
            let existing_tick = events.get(&updated_entity).unwrap();
            if sequence_greater_than(server_tick, *existing_tick) {
                events.insert(updated_entity, server_tick);
            }
        }
    }

    if events.is_empty() {
        return;
    }

    for (updated_entity, update_tick) in &events {
        let Ok((
            net_tile_target,
            mut tile_movement,
            mut physics,
            mut render_position,
            mut animation_state,
            unit_handle,
        )) = updated_q.get_mut(*updated_entity)
        else {
            continue;
        };

        let must_handle_late_update = tile_movement.recv_updated_net_tile_target(
            *update_tick,
            &net_tile_target,
            &mut physics,
            &mut render_position,
        );
        if must_handle_late_update {
            tile_movement.handle_late_update(
                &asset_manager,
                &tick_tracker,
                *update_tick,
                &mut physics,
                &mut render_position,
                &mut animation_state,
                &unit_handle,
            )
        }

        let Ok(mut tick_skipper) = tick_skipper_q.get_mut(*updated_entity) else {
            panic!(
                "failed to get tick_skipper q for entity: {:?}",
                updated_entity
            );
        };
        // info!("entity: {:?}, queueing skipped tick: {:?}", updated_entity, update_tick);
        tick_skipper.queue_skipped_tick(*update_tick);
    }

    rollback_manager.add_events(events);
}

pub fn remove_net_tile_target_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<NetworkedTileTarget>>,
) {
    for _event in event_reader.read() {
        info!("removed NetworkedTileTarget component from entity");
    }

    // TODO: trigger Rollback!
}
