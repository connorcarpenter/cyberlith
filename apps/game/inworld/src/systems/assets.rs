use bevy_ecs::{event::EventReader, system::Commands};

use game_engine::{
    asset::{
        AssetHandle, AssetType, UnitData,
    },
    logging::info,
    world::{
        components::{Main},
        WorldInsertAssetRefEvent,
    },
};

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
            AssetType::Unit => {
                commands
                    .entity(entity)
                    .insert(AssetHandle::<UnitData>::new(asset_id));
            }
            AssetType::Ui => {
                panic!("should not be inserting Ui this way");
            }
            _ => {
                panic!("unexpected asset type for asset ref");
            }
        }

        if AssetType::Unit == asset_type {
            // added AssetHandle component above
            info!("entity {:?} : received Model asset", entity);
        } else {
            panic!("unexpected asset type");
        }
    }
}