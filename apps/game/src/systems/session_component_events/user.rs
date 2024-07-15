use bevy_app::{App, Plugin, Update};
use bevy_ecs::{
    change_detection::ResMut,
    event::{EventReader, EventWriter},
};

use game_engine::{
    logging::info,
    session::{
        components::User, SessionInsertComponentEvent, SessionRemoveComponentEvent,
        SessionUpdateComponentEvent,
    },
};

use crate::{resources::user_manager::UserManager, ui::events::ResyncUserUiEvent};

pub struct UserComponentEventsPlugin;

impl Plugin for UserComponentEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, recv_inserted_user_component)
            .add_systems(Update, recv_updated_user_component)
            .add_systems(Update, recv_removed_user_component);

        // TODO: UserSelf component
        // TODO: UserLobbyOwner component
        // TODO: UserLobbyPeer component
    }
}

fn recv_inserted_user_component(
    mut user_manager: ResMut<UserManager>,
    mut insert_user_event_reader: EventReader<SessionInsertComponentEvent<User>>,
    mut resync_user_ui_event_writer: EventWriter<ResyncUserUiEvent>,
) {
    for event in insert_user_event_reader.read() {
        info!(
            "received Inserted User from Session Server! (entity: {:?})",
            event.entity
        );

        // let user_info = users_q.get(event.entity).unwrap();
        // let user_name = &*user_info.name;
        //
        // info!("incoming user: [ entity({:?}), name({:?}) ]", event.entity, user_name);

        user_manager.insert_user(&mut resync_user_ui_event_writer, event.entity);
    }
}

fn recv_updated_user_component(
    mut update_user_component_event_reader: EventReader<SessionUpdateComponentEvent<User>>,
    mut resync_user_ui_event_writer: EventWriter<ResyncUserUiEvent>,
) {
    for event in update_user_component_event_reader.read() {
        info!(
            "received Updated User from Session Server! (entity: {:?})",
            event.entity
        );

        resync_user_ui_event_writer.send(ResyncUserUiEvent);
    }
}

fn recv_removed_user_component(
    mut user_manager: ResMut<UserManager>,
    mut remove_user_component_event_reader: EventReader<SessionRemoveComponentEvent<User>>,
    mut resync_user_ui_event_writer: EventWriter<ResyncUserUiEvent>,
) {
    for event in remove_user_component_event_reader.read() {
        info!(
            "received Removed User from Session Server! (entity: {:?})",
            event.entity
        );

        user_manager.delete_user(&mut resync_user_ui_event_writer, &event.entity);
    }
}
