use bevy_ecs::schedule::{NextState, State};
use bevy_ecs::system::{Res, ResMut};
use bevy_ecs::{event::EventReader, system::Commands};


use game_engine::asset::{AssetLoadedEvent};
use game_engine::ui::{UiManager};
use game_engine::{
    asset::{
        AnimationData, AssetHandle, AssetType, IconData, MeshData, ModelData, PaletteData,
        SceneData, SkeletonData, SkinData,
    },
    logging::info,
    math::{Quat, Vec3},
    render::components::{RenderLayers, Transform, Visibility},
    world::{
        Alt1, Main, Position, WorldInsertAssetRefEvent, WorldInsertComponentEvent,
        WorldSpawnEntityEvent,
    },
};

use crate::states::AppState;
use crate::systems::walker_scene::{WalkAnimation, WalkerMarker};
use crate::ui::{on_ui_load, UiCatalog};

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
            "received Insert Position from World Server! (entity: {:?})",
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

pub fn session_load_asset_events(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ui_manager: ResMut<UiManager>,
    mut ui_catalog: ResMut<UiCatalog>,
    mut event_reader: EventReader<AssetLoadedEvent>,
) {
    for event in event_reader.read() {
        let asset_id = event.asset_id;
        let asset_type = event.asset_type;
        info!("received Asset Loaded Event! (asset_id: {:?})", asset_id);
        if asset_type == AssetType::Ui {
            let state = *state.get();
            on_ui_load(state, &mut next_state, &mut ui_manager, &mut ui_catalog, asset_id);
        }
    }
}
