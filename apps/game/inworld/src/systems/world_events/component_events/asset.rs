use bevy_ecs::{event::EventReader, prelude::Commands, change_detection::ResMut};

use game_engine::{logging::info, asset::{AssetType, AssetHandle, UnitData}, world::{components::Main, WorldInsertAssetRefEvent}};

use crate::{systems::world_events::PredictionEvents};

pub fn insert_asset_ref_events(
    mut commands: Commands,
    mut prediction_events: ResMut<PredictionEvents>,
    mut insert_asset_ref_events: EventReader<WorldInsertAssetRefEvent<Main>>,
) {
    for event in insert_asset_ref_events.read() {
        let entity = event.entity;
        let asset_type = event.asset_type;
        let asset_id = event.asset_id;

        info!(
            "processing for entity: {:?} = inserting AssetRef<Main>(asset_id: {:?}) ",
            entity, asset_id
        );

        match asset_type {
            AssetType::Unit => {

                info!("entity {:?} : received Unit asset", entity);

                prediction_events.read_insert_unit_asset_ref_event(&entity);

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
    }
}