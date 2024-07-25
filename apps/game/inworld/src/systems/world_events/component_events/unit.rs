use bevy_ecs::{event::EventReader, prelude::{Added, Commands, Query}, entity::Entity, change_detection::ResMut};

use game_engine::{logging::info, asset::{AssetType, AssetHandle, UnitData}, world::{components::Main, WorldInsertAssetRefEvent}};

use crate::{resources::Global};

pub fn late_unit_handle_add(
    mut commands: Commands,
    mut global: ResMut<Global>,
    unit_added_q: Query<(Entity, &AssetHandle<UnitData>), Added<AssetHandle<UnitData>>>
) {
    for (entity, asset_handle) in unit_added_q.iter() {
        info!("AssetHandle<UnitData> component was added to entity {:?}", entity);
        if let Some(owned_entity) = &global.owned_entity {
            if entity == owned_entity.confirmed {

                info!("AssetHandle<UnitData> component was added to confirmed entity {:?}", entity);

                if !global.owned_prediction_has_unit {

                    let predicted_entity = owned_entity.predicted;

                    global.owned_prediction_has_unit = true;

                    info!("now adding AssetHandle<UnitData> component to predicted entity {:?}", predicted_entity);

                    commands.entity(predicted_entity).insert(asset_handle.clone());
                }
            }
        }
    }
}

pub fn main_unit_handle_add(
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
            info!("entity {:?} : received Unit asset", entity);
        } else {
            panic!("unexpected asset type");
        }
    }
}