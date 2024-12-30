use bevy_ecs::{event::EventReader, prelude::Commands, system::ResMut};

use game_engine::{
    asset::{AssetHandle, AssetType, UnitData},
    logging::info,
};

use game_app_network::world::{components::Main, WorldClient, WorldInsertAssetRefEvent};

use crate::resources::RollbackManager;

pub fn insert_asset_ref_events(
    client: WorldClient,
    mut rollback_manager: ResMut<RollbackManager>,
    mut commands: Commands,
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

                let unit_data_handle = AssetHandle::<UnitData>::new(asset_id);

                // prediction_events.read_insert_unit_asset_ref_event(
                //     &now,
                //     &entity,
                //     &unit_data_handle,
                // );
                let server_tick = client.server_tick().unwrap();
                rollback_manager.add_event(entity, server_tick);

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
