use bevy_ecs::{prelude::{Added, Commands, Query}, entity::Entity, change_detection::ResMut};

use game_engine::{logging::info, asset::{AssetHandle, UnitData}};

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