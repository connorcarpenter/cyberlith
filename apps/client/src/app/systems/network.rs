
use bevy_ecs::{
    event::EventReader,
    system::Commands,
};
use bevy_log::info;

use game_engine::{
    world::{Alt1, Position, WorldSpawnEntityEvent, Main, WorldInsertComponentEvent, WorldInsertAssetRefEvent},
    asset::{AnimationData, AssetHandle, AssetType, IconData, MeshData, ModelData, PaletteData, SceneData, SkeletonData, SkinData},
    math::{Quat, Vec3},
    render::components::{RenderLayers, Transform, Visibility}
};
use game_engine::asset::TextStyle;
use game_engine::math::Vec2;

// use crate::app::{systems::scene::{WalkerMarker, WalkAnimation}};
use crate::app::systems::scene::TextMarker;

pub fn world_spawn_entity_events(
    mut event_reader: EventReader<WorldSpawnEntityEvent>,
) {
    for event in event_reader.read() {
        info!("received Spawn Entity from World Server! (entity: {:?})", event.entity);
    }
}

pub fn world_main_insert_position_events(
    mut event_reader: EventReader<WorldInsertComponentEvent<Position>>,
) {
    for event in event_reader.read() {
        info!("received Insert Position from World Server! (entity: {:?})", event.entity);
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

        info!("processing for entity: {:?} = inserting AssetRef<Main>(asset_id: {:?}) ", entity, asset_id);

        match asset_type {
            AssetType::Skeleton => {commands.entity(entity).insert(AssetHandle::<SkeletonData>::new(asset_id));},
            AssetType::Mesh => {commands.entity(entity).insert(AssetHandle::<MeshData>::new(asset_id));},
            AssetType::Palette => {commands.entity(entity).insert(AssetHandle::<PaletteData>::new(asset_id));},
            AssetType::Animation => {commands.entity(entity).insert(AssetHandle::<AnimationData>::new(asset_id));},
            AssetType::Icon => {commands.entity(entity).insert(AssetHandle::<IconData>::new(asset_id));},
            AssetType::Skin => {commands.entity(entity).insert(AssetHandle::<SkinData>::new(asset_id));},
            AssetType::Model => {commands.entity(entity).insert(AssetHandle::<ModelData>::new(asset_id));},
            AssetType::Scene => {commands.entity(entity).insert(AssetHandle::<SceneData>::new(asset_id));},
        }

        // if AssetType::Model == asset_type {
        //     // add clientside things
        //     let layer = RenderLayers::layer(0);
        //
        //     commands
        //         .entity(entity)
        //         .insert(
        //             Transform::from_translation(Vec3::splat(0.0))
        //                 .with_rotation(Quat::from_rotation_z(f32::to_radians(0.0))),
        //         )
        //         .insert(Visibility::default())
        //         // .insert(WalkerMarker)
        //         .insert(layer);
        // }
        // else
        if AssetType::Icon == asset_type {
            // add clientside things
            let layer = RenderLayers::layer(0);

            commands
                .entity(entity)
                .insert(
                    Transform::from_translation_2d(Vec2::splat(64.0)),
                )
                .insert(Visibility::default())
                .insert(TextMarker)
                .insert(TextStyle::new(32.0, 4.0))
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

        info!("processing for entity: {:?} = inserting AssetRef<Alt1>(asset_id: {:?}) ", entity, asset_id);

        if AssetType::Animation == asset_type {
            // let walk_anim = WalkAnimation::new(AssetHandle::<AnimationData>::new(asset_id));
            // commands.entity(entity).insert(walk_anim);
        } else {
            panic!("unexpected asset type");
        }
    }
}