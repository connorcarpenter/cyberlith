use bevy_ecs::{change_detection::ResMut, event::EventReader, prelude::Commands};

use game_engine::{
    asset::{AssetHandle, AssetType, UnitData},
    logging::info,
    time::Instant,
};

use game_app_network::world::{components::Main, WorldInsertAssetRefEvent};

use crate::systems::world_events::PredictionEvents;

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

                let now = Instant::now();
                let unit_data_handle = AssetHandle::<UnitData>::new(asset_id);

                prediction_events.read_insert_unit_asset_ref_event(
                    &now,
                    &entity,
                    &unit_data_handle,
                );

                commands.entity(entity).insert(unit_data_handle);
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
