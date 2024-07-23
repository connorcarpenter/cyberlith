use bevy_ecs::{entity::Entity, event::EventReader, prelude::{NextState, Added}, system::{Commands, Query, ResMut}};

use game_engine::{
    math::{Quat, Vec3},
    logging::{info, warn},
    render::{base::{CpuMaterial, CpuMesh}, components::{Transform, Visibility, RenderLayer, RenderLayers}},
    storage::Storage,
    ui::UiManager,
    naia::{sequence_greater_than, Tick, Replicate},
    asset::{AssetHandle, ModelData},
    world::{
        WorldClient,
        behavior as shared_behavior,
        channels::{EntityAssignmentChannel, PlayerCommandChannel},
        messages::{EntityAssignment, KeyCommand},
        components::Position, WorldConnectEvent, WorldRemoveComponentEvent, WorldClientTickEvent,
        WorldInsertComponentEvent, WorldSpawnEntityEvent, WorldDespawnEntityEvent, WorldDisconnectEvent, WorldMessageEvents, WorldRejectEvent, WorldUpdateComponentEvent,
    },
};

use game_app_common::AppState;

use crate::{systems::scene_setup, resources::{Global, OwnedEntity}, components::{Interp, Confirmed, WalkAnimation, Predicted}};

pub fn connect_events(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut meshes: ResMut<Storage<CpuMesh>>,
    mut materials: ResMut<Storage<CpuMaterial>>,
    mut ui_manager: ResMut<UiManager>,
    render_layer_q: Query<(Entity, &RenderLayer)>,

    mut event_reader: EventReader<WorldConnectEvent>,
) {
    // this one loop will only run once
    for _ in event_reader.read() {
        info!("received Connect to World Server!");

        // despawning all entities with RenderLayer(0)
        let render_layer_0 = RenderLayers::layer(0);
        for (entity, layer) in render_layer_q.iter() {
            if *layer == render_layer_0 {
                commands.entity(entity).despawn();
            }
        }

        // setup walker scene
        scene_setup::scene_setup(
            &mut commands,
            &mut meshes,
            &mut materials,
        );

        // disable ui
        ui_manager.disable_ui();

        // set to appropriate AppState
        next_state.set(AppState::InGame);
        return;
    }
}

pub fn reject_events(
    mut event_reader: EventReader<WorldRejectEvent>
) {
    for _ in event_reader.read() {
        info!("Client rejected from connecting to Server");
    }
}

pub fn disconnect_events(
    mut event_reader: EventReader<WorldDisconnectEvent>
) {
    for _ in event_reader.read() {
        info!("Client disconnected from Server");
    }
}

pub fn message_events(
    mut commands: Commands,
    client: WorldClient,
    mut global: ResMut<Global>,
    mut event_reader: EventReader<WorldMessageEvents>,
    position_q: Query<&Position>,
    model_q: Query<&AssetHandle<ModelData>>,
    walk_animation_q: Query<&WalkAnimation>,
) {
    for events in event_reader.read() {
        for message in events.read::<EntityAssignmentChannel, EntityAssignment>() {
            let assign = message.assign;

            let entity = message.entity.get(&client).unwrap();
            if assign {
                info!("gave ownership of entity: {:?}", entity);

                // Here we create a local copy of the Player entity, to use for client-side prediction
                if let Ok(position) = position_q.get(entity) {

                    let mut owned_has_animation: bool = false;
                    let walk_animation_opt = {
                        if let Ok(walk_animation) = walk_animation_q.get(entity) {
                            let walk_animation: WalkAnimation = walk_animation.clone();
                            owned_has_animation = true;
                            Some(walk_animation)
                        } else {
                            warn!("entity {:?} does not have WalkAnimation component yet!", entity);
                            None
                        }
                    };

                    let mut owned_has_model: bool = false;
                    let model_opt = {
                        if let Ok(model) = model_q.get(entity) {
                            let model: AssetHandle<ModelData> = model.clone();
                            owned_has_model = true;
                            Some(model)
                        } else {
                            warn!("entity {:?} does not have AssetHandle<ModelData> component yet!", entity);
                            None
                        }
                    };

                    let prediction_entity = commands
                        .spawn_empty()
                        .id();
                    let mut prediction_position = Position::new(*position.x, *position.y);
                    prediction_position.localize();
                    commands
                        .entity(prediction_entity)
                        .insert(prediction_position)
                        .insert(RenderLayers::layer(0))
                        .insert(Visibility::default())
                        .insert(Transform::default())
                        // insert interpolation component
                        .insert(Interp::new(*position.x, *position.y))
                        // mark as predicted
                        .insert(Predicted);

                    if let Some(walk_animation) = walk_animation_opt {
                        commands
                            .entity(prediction_entity)
                            .insert(walk_animation);
                    }
                    if let Some(model) = model_opt {
                        commands
                            .entity(prediction_entity)
                            .insert(model);
                    }

                    info!(
                        "from confirmed entity: {:?}, created prediction entity: {:?}, has model: {:?}, has animation: {:?}",
                        entity,
                        prediction_entity,
                        owned_has_model,
                        owned_has_animation
                    );
                    global.owned_entity = Some(OwnedEntity::new(entity, prediction_entity));
                    global.owned_prediction_has_model = owned_has_model;
                    global.owned_prediction_has_animation = owned_has_animation;
                } else {
                    warn!("entity {:?} does not have Position component yet!", entity);
                }
            } else {
                let mut disowned: bool = false;
                if let Some(owned_entity) = &global.owned_entity {
                    if owned_entity.confirmed == entity {
                        commands.entity(owned_entity.predicted).despawn();
                        disowned = true;
                    }
                }
                if disowned {
                    info!("removed ownership of entity");
                    global.owned_entity = None;
                }
            }
        }
    }
}

pub fn late_model_handle_add(
    mut commands: Commands,
    mut global: ResMut<Global>,
    model_added_q: Query<(Entity, &AssetHandle<ModelData>), Added<AssetHandle<ModelData>>>
) {
    for (entity, asset_handle) in model_added_q.iter() {
        info!("AssetHandle<ModelData> component was added to entity {:?}", entity);
        if let Some(owned_entity) = &global.owned_entity {
            if entity == owned_entity.confirmed {

                info!("AssetHandle<ModelData> component was added to confirmed entity {:?}", entity);

                if !global.owned_prediction_has_model {

                    let predicted_entity = owned_entity.predicted;

                    global.owned_prediction_has_model = true;

                    info!("now adding AssetHandle<ModelData> component to predicted entity {:?}", predicted_entity);

                    commands.entity(predicted_entity).insert(asset_handle.clone());
                }
            }
        }
    }
}

pub fn late_animation_handle_add(
    mut commands: Commands,
    mut global: ResMut<Global>,
    animation_added_q: Query<(Entity, &WalkAnimation), Added<WalkAnimation>>
) {
    for (entity, walk_animation) in animation_added_q.iter() {
        info!("WalkAnimation component was added to entity {:?}", entity);
        if let Some(owned_entity) = &global.owned_entity {
            if entity == owned_entity.confirmed {

                info!("WalkAnimation component was added to confirmed entity {:?}", entity);

                if !global.owned_prediction_has_animation {

                    let predicted_entity = owned_entity.predicted;

                    global.owned_prediction_has_animation = true;

                    info!("now adding WalkAnimation component to predicted entity {:?}", predicted_entity);

                    commands.entity(predicted_entity).insert(walk_animation.clone());
                }
            }
        }
    }
}

pub fn spawn_entity_events(
    mut event_reader: EventReader<WorldSpawnEntityEvent>
) {
    for event in event_reader.read() {
        info!(
            "received Spawn Entity from World Server! (entity: {:?})",
            event.entity
        );
    }
}

pub fn despawn_entity_events(
    mut event_reader: EventReader<WorldDespawnEntityEvent>
) {
    for event in event_reader.read() {
        info!(
            "received Despawn Entity from World Server! (entity: {:?})",
            event.entity
        );
    }
}

pub fn insert_position_events(
    mut commands: Commands,
    position_q: Query<&Position>,
    mut event_reader: EventReader<WorldInsertComponentEvent<Position>>,
) {
    for event in event_reader.read() {
        let entity = event.entity;

        info!(
            "received Inserted Component: `Position` from World Server! (entity: {:?})",
            entity
        );
        if let Ok(position) = position_q.get(entity) {

            let layer = RenderLayers::layer(0);

            commands
                .entity(entity)
                .insert(layer)
                .insert(Visibility::default())
                .insert(
                    Transform::from_translation(Vec3::splat(0.0))
                        .with_rotation(Quat::from_rotation_z(f32::to_radians(90.0))),
                )
                // initialize interpolation
                .insert(Interp::new(*position.x, *position.y))
                .insert(Confirmed);
        } else {
            warn!("entity does not have Position component");
        }
    }
}

pub fn update_position_events(
    mut global: ResMut<Global>,
    mut event_reader: EventReader<WorldUpdateComponentEvent<Position>>,
    mut position_query: Query<&mut Position>,
) {
    // When we receive a new Position update for the Player's Entity,
    // we must ensure the Client-side Prediction also remains in-sync
    // So we roll the Prediction back to the authoritative Server state
    // and then execute all Player Commands since that tick, using the CommandHistory helper struct
    if let Some(owned_entity) = &global.owned_entity {
        let mut latest_tick: Option<Tick> = None;
        let server_entity = owned_entity.confirmed;
        let client_entity = owned_entity.predicted;

        for event in event_reader.read() {
            let server_tick = event.tick;
            let updated_entity = event.entity;

            // If entity is owned
            if updated_entity == server_entity {
                if let Some(last_tick) = &mut latest_tick {
                    if sequence_greater_than(server_tick, *last_tick) {
                        *last_tick = server_tick;
                    }
                } else {
                    latest_tick = Some(server_tick);
                }
            }
        }

        if let Some(server_tick) = latest_tick {
            if let Ok([server_position, mut client_position]) =
                position_query.get_many_mut([server_entity, client_entity])
            {
                // Set to authoritative state
                client_position.mirror(&*server_position);

                // Replay all stored commands

                // TODO: why is it necessary to subtract 1 Tick here?
                // it's not like this in the Macroquad demo
                let modified_server_tick = server_tick.wrapping_sub(1);

                let replay_commands = global.command_history.replays(&modified_server_tick);
                for (_command_tick, command) in replay_commands {
                    shared_behavior::process_command(&command, &mut client_position);
                }
            }
        }
    }
}

pub fn remove_position_events(
    mut event_reader: EventReader<WorldRemoveComponentEvent<Position>>
) {
    for _event in event_reader.read() {
        info!("removed Position component from entity");
    }
}

pub fn tick_events(
    mut client: WorldClient,
    mut global: ResMut<Global>,
    mut tick_reader: EventReader<WorldClientTickEvent>,
    mut position_q: Query<&mut Position>,
) {
    let Some(predicted_entity) = global
        .owned_entity
        .as_ref()
        .map(|owned_entity| owned_entity.predicted)
    else {
        // No owned Entity
        return;
    };

    let Some(command) = global.queued_command.take() else {
        return;
    };

    for event in tick_reader.read() {
        let client_tick = event.tick;

        // Command History
        if !global.command_history.can_insert(&client_tick) {
            // History is full
            continue;
        }

        // Record command
        global.command_history.insert(client_tick, command.clone());

        // Send command
        client.send_tick_buffer_message::<PlayerCommandChannel, KeyCommand>(&client_tick, &command);

        if let Ok(mut position) = position_q.get_mut(predicted_entity) {
            // Apply command
            shared_behavior::process_command(&command, &mut position);
        }
    }
}