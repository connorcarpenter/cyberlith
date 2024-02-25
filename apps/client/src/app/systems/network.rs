
use bevy_ecs::{
    prelude::Query,
    event::EventReader,
    system::{Commands, ResMut},
};
use bevy_log::info;

use game_engine::{world::{Alt1, WorldSpawnEntityEvent, WorldClient, WorldConnectEvent, AssetEntry, AssetRef, Main, WorldInsertComponentEvents}, asset::{AssetCache, AssetMetadataStore}, math::{Quat, Vec3}, render::components::{RenderLayers, Transform, Visibility}, ConnectionManager};

use crate::app::{systems::scene::ObjectMarker, resources::{asset_ref_processor::AssetRefProcessor}};

pub fn world_connect_events(
    client: WorldClient,
    mut event_reader: EventReader<WorldConnectEvent>,
    mut connection_manager: ResMut<ConnectionManager>,
) {
    for _ in event_reader.read() {
        let Ok(server_address) = client.server_address() else {
            panic!("Shouldn't happen");
        };
        info!(
            "Client connected to world server at addr: {}",
            server_address
        );

        connection_manager.handle_world_connection_event();
    }
}

pub fn world_spawn_entity_events(
    mut event_reader: EventReader<WorldSpawnEntityEvent>,
) {
    for events in event_reader.read() {
        info!("received Spawn Entity from World Server! (entity: {:?})", events.entity);
    }
}

// most likely will need to just split this up into individual insert component systems like in editor
pub fn world_insert_component_events(
    mut commands: Commands,
    client: WorldClient,
    mut event_reader: EventReader<WorldInsertComponentEvents>,
    mut metadata_store: ResMut<AssetMetadataStore>,
    asset_cache: ResMut<AssetCache>,
    mut asset_ref_processor: ResMut<AssetRefProcessor>,
    asset_entry_q: Query<&AssetEntry>,
    asset_ref_main_q: Query<&AssetRef<Main>>,
    asset_ref_alt1_q: Query<&AssetRef<Alt1>>,
) {
    for events in event_reader.read() {
        for entity in events.read::<AssetEntry>() {
            let Ok(asset_entry) = asset_entry_q.get(entity) else {
                panic!("Shouldn't happen");
            };
            let asset_id = *asset_entry.asset_id;
            info!("received Asset Entry from World Server! (entity: {:?}, asset_id: {:?})", entity, asset_id);
            asset_ref_processor.handle_add_asset_entry(&mut commands, &mut metadata_store, &asset_cache, &entity, &asset_id);
        }
        for entity in events.read::<AssetRef<Main>>() {
            AssetRefProcessor::insert_asset_ref_events::<Main>(&mut commands, &client, &asset_cache, &mut metadata_store, &mut asset_ref_processor, &asset_entry_q, &asset_ref_main_q, &entity);

            // add clientside things
            let layer = RenderLayers::layer(0);

            // model
            commands
                .entity(entity)
                // .insert(WalkAnimation {
                //     anim_handle: human_walk_anim_handle,
                //     image_index: 0.0,
                // })
                .insert(
                    Transform::from_translation(Vec3::splat(0.0))
                        .with_rotation(Quat::from_rotation_z(f32::to_radians(0.0))),
                )
                .insert(Visibility::default())
                .insert(ObjectMarker)
                .insert(layer);
        }
        for entity in events.read::<AssetRef<Alt1>>() {
            AssetRefProcessor::insert_asset_ref_events::<Alt1>(&mut commands, &client, &asset_cache, &mut metadata_store, &mut asset_ref_processor, &asset_entry_q, &asset_ref_alt1_q, &entity);
        }
        // .. other components here later
    }
}