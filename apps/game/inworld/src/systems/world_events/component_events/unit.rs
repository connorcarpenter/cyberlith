use bevy_ecs::{prelude::{Added, Commands, Query}, entity::Entity, change_detection::ResMut};

use game_engine::{logging::info, asset::{AssetHandle, ModelData}};

use crate::{resources::Global, components::WalkAnimation};

pub fn late_model_handle_add(
    mut commands: Commands,
    mut global: ResMut<Global>,
    model_added_q: Query<(Entity, &AssetHandle<ModelData>), Added<AssetHandle<ModelData>>>
) {
    for (entity, asset_handle) in model_added_q.iter() {
        info!("AssetHandle<ModelData> component was added to entity {:?}", entity);
        if let Some(owned_entity) = &global.owned_entity {
            if entity == owned_entity.confirmed {

                info!("AssetHandle<ModelData> component was added to confirmed entity {:?}", entity);

                if !global.owned_prediction_has_model {

                    let predicted_entity = owned_entity.predicted;

                    global.owned_prediction_has_model = true;

                    info!("now adding AssetHandle<ModelData> component to predicted entity {:?}", predicted_entity);

                    commands.entity(predicted_entity).insert(asset_handle.clone());
                }
            }
        }
    }
}

pub fn late_animation_handle_add(
    mut commands: Commands,
    mut global: ResMut<Global>,
    animation_added_q: Query<(Entity, &WalkAnimation), Added<WalkAnimation>>
) {
    for (entity, walk_animation) in animation_added_q.iter() {
        info!("WalkAnimation component was added to entity {:?}", entity);
        if let Some(owned_entity) = &global.owned_entity {
            if entity == owned_entity.confirmed {

                info!("WalkAnimation component was added to confirmed entity {:?}", entity);

                if !global.owned_prediction_has_animation {

                    let predicted_entity = owned_entity.predicted;

                    global.owned_prediction_has_animation = true;

                    info!("now adding WalkAnimation component to predicted entity {:?}", predicted_entity);

                    commands.entity(predicted_entity).insert(walk_animation.clone());
                }
            }
        }
    }
}