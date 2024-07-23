use bevy_ecs::{event::EventReader, system::Commands};

use game_engine::{
    asset::{
        AnimationData, AssetHandle, AssetType, IconData, MeshData, ModelData, PaletteData,
        SceneData, SkeletonData, SkinData,
    },
    logging::info,
    world::{
        components::{Alt1, Main},
        WorldInsertAssetRefEvent,
    },
};

use crate::components::WalkAnimation;

pub fn main_insert_asset_ref_events(
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
            // added AssetHandle component above
            info!("entity {:?} : received Model asset", entity);
        } else {
            panic!("unexpected asset type");
        }
    }
}

pub fn alt1_insert_asset_ref_events(
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
            info!("adding WalkAnimation to entity: {:?}", entity);
            let walk_anim = WalkAnimation::new(AssetHandle::<AnimationData>::new(asset_id));
            commands.entity(entity).insert(walk_anim);
        } else {
            panic!("unexpected asset type");
        }
    }
}
