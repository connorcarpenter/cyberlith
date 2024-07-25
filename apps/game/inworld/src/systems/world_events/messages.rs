use bevy_ecs::{prelude::{Commands, Query}, event::EventReader, change_detection::ResMut};

use game_engine::{
    naia::Replicate,
    world::{messages::EntityAssignment, components::Position, WorldClient, WorldMessageEvents, channels::EntityAssignmentChannel},
    logging::{info, warn},
    asset::{AssetHandle, UnitData}, render::components::{RenderLayers, Transform, Visibility}};

use crate::{resources::{Global, OwnedEntity}, components::{Interp, Predicted, AnimationState}};

pub fn message_events(
    mut commands: Commands,
    client: WorldClient,
    mut global: ResMut<Global>,
    mut event_reader: EventReader<WorldMessageEvents>,
    position_q: Query<&Position>,
    unit_q: Query<&AssetHandle<UnitData>>,
) {
    for events in event_reader.read() {
        for message in events.read::<EntityAssignmentChannel, EntityAssignment>() {
            let assign = message.assign;

            let entity = message.entity.get(&client).unwrap();
            if assign {
                info!("gave ownership of entity: {:?}", entity);

                // Here we create a local copy of the Player entity, to use for client-side prediction
                if let Ok(position) = position_q.get(entity) {

                    let mut owned_has_unit: bool = false;
                    let unit_opt = {
                        if let Ok(unit_data) = unit_q.get(entity) {
                            let model: AssetHandle<UnitData> = unit_data.clone();
                            owned_has_unit = true;
                            Some(model)
                        } else {
                            warn!("entity {:?} does not have AssetHandle<UnitData> component yet!", entity);
                            None
                        }
                    };

                    let prediction_entity = commands
                        .spawn_empty()
                        .id();
                    let mut prediction_position = Position::new(*position.x, *position.y);
                    prediction_position.localize();
                    commands
                        .entity(prediction_entity)
                        .insert(prediction_position)
                        .insert(RenderLayers::layer(0))
                        .insert(Visibility::default())
                        .insert(Transform::default())
                        .insert(AnimationState::new())
                        // insert interpolation component
                        .insert(Interp::new(*position.x, *position.y))
                        // mark as predicted
                        .insert(Predicted);

                    if let Some(unit) = unit_opt {
                        commands
                            .entity(prediction_entity)
                            .insert(unit);
                    }

                    info!(
                        "from confirmed entity: {:?}, created prediction entity: {:?}, has unit: {:?}",
                        entity,
                        prediction_entity,
                        owned_has_unit,
                    );
                    global.owned_entity = Some(OwnedEntity::new(entity, prediction_entity));
                    global.owned_prediction_has_unit = owned_has_unit;
                } else {
                    warn!("entity {:?} does not have Position component yet!", entity);
                }
            } else {
                let mut disowned: bool = false;
                if let Some(owned_entity) = &global.owned_entity {
                    if owned_entity.confirmed == entity {
                        commands.entity(owned_entity.predicted).despawn();
                        disowned = true;
                    }
                }
                if disowned {
                    info!("removed ownership of entity");
                    global.owned_entity = None;
                }
            }
        }
    }
}