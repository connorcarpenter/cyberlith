use bevy_ecs::{change_detection::ResMut, event::EventReader, prelude::Commands};

use game_engine::logging::info;

use game_app_network::world::{
    channels::EntityAssignmentChannel, messages::EntityAssignment, WorldClient, WorldMessageEvents,
};

use crate::resources::{Global, RollbackManager};

pub fn message_events(
    mut commands: Commands,
    client: WorldClient,
    mut global: ResMut<Global>,
    mut rollback_manager: ResMut<RollbackManager>,
    mut message_events: EventReader<WorldMessageEvents>,
) {
    for events in message_events.read() {
        for message in events.read::<EntityAssignmentChannel, EntityAssignment>() {

            let server_tick = client.server_tick().unwrap();
            let assign = message.assign;

            let entity = message.entity.get(&client).unwrap();
            if assign {
                info!("gave ownership of entity: {:?}", entity);
                global.owned_entity = Some(entity);
                rollback_manager.add_event(entity, server_tick);
            } else {
                let mut disowned: bool = false;
                if let Some(owned_entity) = &global.owned_entity {
                    if *owned_entity == entity {
                        commands.entity(*owned_entity).despawn();
                        disowned = true;
                    }
                }
                if disowned {
                    info!("removed ownership of entity");
                    global.owned_entity = None;
                    rollback_manager.add_event(entity, server_tick);
                }
            }
        }
    }
}
