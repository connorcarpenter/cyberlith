use bevy_ecs::{prelude::{Commands, Query}, event::EventReader, change_detection::ResMut};

use game_engine::{naia::Replicate, world::{messages::EntityAssignment, components::Position, WorldClient, WorldMessageEvents, channels::EntityAssignmentChannel}, logging::{info, warn}, asset::{AssetHandle, ModelData}, render::components::{RenderLayers, Transform, Visibility}};

use crate::{resources::{Global, OwnedEntity}, components::{Interp, Predicted, WalkAnimation}};

pub fn message_events(
    mut commands: Commands,
    client: WorldClient,
    mut global: ResMut<Global>,
    mut event_reader: EventReader<WorldMessageEvents>,
    position_q: Query<&Position>,
    model_q: Query<&AssetHandle<ModelData>>,
    walk_animation_q: Query<&WalkAnimation>,
) {
    for events in event_reader.read() {
        for message in events.read::<EntityAssignmentChannel, EntityAssignment>() {
            let assign = message.assign;

            let entity = message.entity.get(&client).unwrap();
            if assign {
                info!("gave ownership of entity: {:?}", entity);

                // Here we create a local copy of the Player entity, to use for client-side prediction
                if let Ok(position) = position_q.get(entity) {

                    let mut owned_has_animation: bool = false;
                    let walk_animation_opt = {
                        if let Ok(walk_animation) = walk_animation_q.get(entity) {
                            let walk_animation: WalkAnimation = walk_animation.clone();
                            owned_has_animation = true;
                            Some(walk_animation)
                        } else {
                            warn!("entity {:?} does not have WalkAnimation component yet!", entity);
                            None
                        }
                    };

                    let mut owned_has_model: bool = false;
                    let model_opt = {
                        if let Ok(model) = model_q.get(entity) {
                            let model: AssetHandle<ModelData> = model.clone();
                            owned_has_model = true;
                            Some(model)
                        } else {
                            warn!("entity {:?} does not have AssetHandle<ModelData> component yet!", entity);
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
                        // insert interpolation component
                        .insert(Interp::new(*position.x, *position.y))
                        // mark as predicted
                        .insert(Predicted);

                    if let Some(walk_animation) = walk_animation_opt {
                        commands
                            .entity(prediction_entity)
                            .insert(walk_animation);
                    }
                    if let Some(model) = model_opt {
                        commands
                            .entity(prediction_entity)
                            .insert(model);
                    }

                    info!(
                        "from confirmed entity: {:?}, created prediction entity: {:?}, has model: {:?}, has animation: {:?}",
                        entity,
                        prediction_entity,
                        owned_has_model,
                        owned_has_animation
                    );
                    global.owned_entity = Some(OwnedEntity::new(entity, prediction_entity));
                    global.owned_prediction_has_model = owned_has_model;
                    global.owned_prediction_has_animation = owned_has_animation;
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