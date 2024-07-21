use bevy_ecs::{entity::Entity, event::EventReader, prelude::NextState, system::{Commands, Query, ResMut}};

use game_engine::{
    asset::{
        AnimationData, AssetHandle, AssetType, IconData, MeshData, ModelData, PaletteData,
        SceneData, SkeletonData, SkinData,
    },
    logging::info,
    math::{Quat, Vec3},
    render::{base::{CpuMaterial, CpuMesh}, components::{RenderLayer, RenderLayers, Transform, Visibility}},
    storage::Storage,
    ui::UiManager,
    world::{
        components::{Alt1, Main, Position}, WorldConnectEvent,
        WorldInsertAssetRefEvent, WorldInsertComponentEvent, WorldSpawnEntityEvent,
    },
};

use crate::{states::AppState, inworld::systems::{walker_scene, walker_scene::{WalkAnimation, WalkerMarker}}};

pub fn world_connect_events(
    mut commands: Commands,
    mut next_state: ResMut<NextState<AppState>>,
    mut meshes: ResMut<Storage<CpuMesh>>,
    mut materials: ResMut<Storage<CpuMaterial>>,
    mut ui_manager: ResMut<UiManager>,
    render_layer_q: Query<(Entity, &RenderLayer)>,

    mut event_reader: EventReader<WorldConnectEvent>,
) {
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
        walker_scene::scene_setup(
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

pub fn world_spawn_entity_events(mut event_reader: EventReader<WorldSpawnEntityEvent>) {
    for event in event_reader.read() {
        info!(
            "received Spawn Entity from World Server! (entity: {:?})",
            event.entity
        );
    }
}

pub fn world_main_insert_position_events(
    mut event_reader: EventReader<WorldInsertComponentEvent<Position>>,
) {
    for event in event_reader.read() {
        info!(
            "received Inserted Component: `Position` from World Server! (entity: {:?})",
            event.entity
        );
    }
}

pub fn world_main_insert_asset_ref_events(
    mut commands: Commands,
    mut event_reader: EventReader<WorldInsertAssetRefEvent<Main>>,
) {
    for event in event_reader.read() {
        let entity = event.entity;
        let asset_type = event.asset_type;
        let asset_id = event.asset_id;

        info!(
            "processing for entity: {:?} = inserting AssetRef<Main>(asset_id: {:?}) ",
            entity, asset_id
        );

        match asset_type {
            AssetType::Skeleton => {
                commands
                    .entity(entity)
                    .insert(AssetHandle::<SkeletonData>::new(asset_id));
            }
            AssetType::Mesh => {
                commands
                    .entity(entity)
                    .insert(AssetHandle::<MeshData>::new(asset_id));
            }
            AssetType::Palette => {
                commands
                    .entity(entity)
                    .insert(AssetHandle::<PaletteData>::new(asset_id));
            }
            AssetType::Animation => {
                commands
                    .entity(entity)
                    .insert(AssetHandle::<AnimationData>::new(asset_id));
            }
            AssetType::Icon => {
                commands
                    .entity(entity)
                    .insert(AssetHandle::<IconData>::new(asset_id));
            }
            AssetType::Skin => {
                commands
                    .entity(entity)
                    .insert(AssetHandle::<SkinData>::new(asset_id));
            }
            AssetType::Model => {
                commands
                    .entity(entity)
                    .insert(AssetHandle::<ModelData>::new(asset_id));
            }
            AssetType::Scene => {
                commands
                    .entity(entity)
                    .insert(AssetHandle::<SceneData>::new(asset_id));
            }
            AssetType::Ui => {
                panic!("should not be inserting Ui this way");
            }
        }

        if AssetType::Model == asset_type {
            info!("received model");
            // add clientside things
            let layer = RenderLayers::layer(0);

            commands
                .entity(entity)
                .insert(
                    Transform::from_translation(Vec3::splat(0.0))
                        .with_rotation(Quat::from_rotation_z(f32::to_radians(0.0))),
                )
                .insert(Visibility::default())
                .insert(WalkerMarker)
                .insert(layer);
        } else {
            panic!("unexpected asset type");
        }
    }
}

pub fn world_alt1_insert_asset_ref_events(
    mut commands: Commands,
    mut event_reader: EventReader<WorldInsertAssetRefEvent<Alt1>>,
) {
    for event in event_reader.read() {
        let entity = event.entity;
        let asset_type = event.asset_type;
        let asset_id = event.asset_id;

        info!(
            "processing for entity: {:?} = inserting AssetRef<Alt1>(asset_id: {:?}) ",
            entity, asset_id
        );

        if AssetType::Animation == asset_type {
            let walk_anim = WalkAnimation::new(AssetHandle::<AnimationData>::new(asset_id));
            commands.entity(entity).insert(walk_anim);
        } else {
            panic!("unexpected asset type");
        }
    }
}
