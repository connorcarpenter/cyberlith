use bevy_ecs::{change_detection::ResMut, event::EventReader, prelude::Commands};

use game_engine::{
    logging::info,
    time::Instant,
    world::{
        channels::EntityAssignmentChannel, messages::EntityAssignment, WorldClient,
        WorldMessageEvents,
    },
};

use crate::{resources::Global, systems::world_events::PredictionEvents};

pub fn message_events(
    mut commands: Commands,
    client: WorldClient,
    mut global: ResMut<Global>,
    mut prediction_events: ResMut<PredictionEvents>,
    mut message_events: EventReader<WorldMessageEvents>,
) {
    for events in message_events.read() {
        for message in events.read::<EntityAssignmentChannel, EntityAssignment>() {
            let now = Instant::now();
            let assign = message.assign;

            let entity = message.entity.get(&client).unwrap();
            if assign {
                info!("gave ownership of entity: {:?}", entity);

                prediction_events.read_entity_assignment_event(&now, &entity);
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
