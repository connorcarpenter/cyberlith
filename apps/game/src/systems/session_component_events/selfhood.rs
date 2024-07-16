use bevy_app::{App, Plugin, Update};
use bevy_ecs::{prelude::Query, event::{EventReader, EventWriter}, change_detection::ResMut};

use game_engine::{session::{SessionClient, SessionInsertComponentEvent, components::{Selfhood, SelfhoodUser, User}}, logging::info};

use crate::{ui::events::ResyncUserListUiEvent, resources::{user_manager::UserManager, selfhood_events::SelfhoodEvents}};

pub struct SelfhoodComponentEventsPlugin;

impl Plugin for SelfhoodComponentEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, recv_inserted_selfhood_components)
        ;
    }
}

fn recv_inserted_selfhood_components(
    session_client: SessionClient,
    mut user_manager: ResMut<UserManager>,
    mut selfhood_events: ResMut<SelfhoodEvents>,
    mut insert_selfhood_event_reader: EventReader<SessionInsertComponentEvent<Selfhood>>,
    mut insert_selfhood_user_event_reader: EventReader<SessionInsertComponentEvent<SelfhoodUser>>,
    mut resync_user_ui_event_writer: EventWriter<ResyncUserListUiEvent>,
    selfhood_user_q: Query<&SelfhoodUser>,
    user_q: Query<&User>,
) {
    for self_entity in selfhood_events.recv_inserted_component_events(
        &mut insert_selfhood_event_reader,
        &mut insert_selfhood_user_event_reader,
    ) {
        let selfhood_user = selfhood_user_q.get(self_entity).unwrap();
        let user_entity = selfhood_user.user_entity.get(&session_client).unwrap();
        let user = user_q.get(user_entity).unwrap();

        info!(
            "received Inserted Selfhood from Session Server!  [ {:?} ]",
            *user.name,
        );

        user_manager.set_self_user_entity(&mut resync_user_ui_event_writer, user_entity);
    }
}