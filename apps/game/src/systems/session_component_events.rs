use bevy_app::{App, Plugin, Update};
use bevy_ecs::{
    change_detection::ResMut,
    event::{EventReader, EventWriter},
    prelude::Query,
};
use bevy_ecs::system::Res;
use game_engine::{
    logging::info,
    session::{
        components::{LobbyLocal, MessagePublic, UserPublic},
        SessionInsertComponentEvent, SessionRemoveComponentEvent, SessionUpdateComponentEvent,
    },
};

use crate::{
    resources::{message_manager::MessageManager, lobby_manager::LobbyManager, user_manager::UserManager},
    ui::events::{ResyncLobbyGlobalEvent, ResyncMatchLobbiesEvent, ResyncPublicUserInfoEvent},
};

pub struct SessionComponentEventsPlugin;

impl Plugin for SessionComponentEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            // messages
            .add_systems(Update, recv_inserted_message_public_component)

            // lobbies
            .add_systems(Update, recv_inserted_lobby_public_component)
            .add_systems(Update, recv_removed_lobby_public_component)

            // users
            .add_systems(Update, recv_inserted_user_public_info_component)
            .add_systems(Update, recv_updated_user_public_info_component)
            .add_systems(Update, recv_removed_user_public_component);
    }
}

pub fn recv_inserted_user_public_info_component(
    mut user_manager: ResMut<UserManager>,
    mut insert_user_info_event_reader: EventReader<SessionInsertComponentEvent<UserPublic>>,
    mut resync_event_writer: EventWriter<ResyncPublicUserInfoEvent>,
) {
    for event in insert_user_info_event_reader.read() {
        info!(
            "received Inserted PublicUserInfo from Session Server! (entity: {:?})",
            event.entity
        );

        // let user_info = users_q.get(event.entity).unwrap();
        // let user_name = &*user_info.name;
        //
        // info!("incoming user: [ entity({:?}), name({:?}) ]", event.entity, user_name);

        user_manager.insert_user(&mut resync_event_writer, event.entity);
    }
}

pub fn recv_updated_user_public_info_component(
    mut update_user_public_event_reader: EventReader<SessionUpdateComponentEvent<UserPublic>>,
    mut resync_user_public_event_writer: EventWriter<ResyncPublicUserInfoEvent>,
) {
    for event in update_user_public_event_reader.read() {
        info!(
            "received Updated PublicUserInfo from Session Server! (entity: {:?})",
            event.entity
        );

        resync_user_public_event_writer.send(ResyncPublicUserInfoEvent);
    }
}

pub fn recv_removed_user_public_component(
    mut user_manager: ResMut<UserManager>,
    mut resync_user_public_info_event_writer: EventWriter<ResyncPublicUserInfoEvent>,
    mut event_reader: EventReader<SessionRemoveComponentEvent<UserPublic>>,
) {
    for event in event_reader.read() {
        info!(
            "received Removed PublicUserInfo from Session Server! (entity: {:?})",
            event.entity
        );

        user_manager.delete_user(&mut resync_user_public_info_event_writer, &event.entity);
    }
}

pub fn recv_inserted_message_public_component(
    lobby_manager: Res<LobbyManager>,
    mut message_manager: ResMut<MessageManager>,
    mut resync_global_chat_events: EventWriter<ResyncLobbyGlobalEvent>,
    mut message_event_reader: EventReader<SessionInsertComponentEvent<MessagePublic>>,
    message_q: Query<&MessagePublic>,
) {
    for event in message_event_reader.read() {
        // info!("received Inserted GlobalChatMessage from Session Server! (entity: {:?})", event.entity);

        let message = message_q.get(event.entity).unwrap();
        let message_id = *message.id;

        let timestamp = *message.timestamp;
        let message = &*message.message;
        info!(
            "incoming global message: [ {:?} | {:?} | {:?} ]",
            timestamp, event.entity, message
        );

        let lobby_id_opt = lobby_manager.get_current_lobby_id();
        message_manager.recv_message(&lobby_id_opt, &mut resync_global_chat_events, message_id, event.entity);
    }
}

pub fn recv_inserted_lobby_public_component(
    mut lobby_manager: ResMut<LobbyManager>,
    mut resync_match_lobby_events: EventWriter<ResyncMatchLobbiesEvent>,
    lobby_q: Query<&LobbyLocal>,
    mut insert_lobby_event_reader: EventReader<SessionInsertComponentEvent<LobbyLocal>>,
) {
    for event in insert_lobby_event_reader.read() {
        let lobby = lobby_q.get(event.entity).unwrap();
        let lobby_id = *lobby.id;

        let lobby_name = &*lobby.name;
        info!(
            "incoming match lobby: [ {:?} | {:?} ]",
            event.entity, lobby_name
        );

        lobby_manager.recv_lobby(&mut resync_match_lobby_events, lobby_id, event.entity);
    }
}

pub fn recv_removed_lobby_public_component(
    mut lobby_manager: ResMut<LobbyManager>,
    mut resync_match_lobby_events: EventWriter<ResyncMatchLobbiesEvent>,
    mut remove_lobby_event_reader: EventReader<SessionRemoveComponentEvent<LobbyLocal>>,
) {
    for event in remove_lobby_event_reader.read() {
        info!(
            "received Removed MatchLobby from Session Server! (entity: {:?})",
            event.entity
        );

        lobby_manager.remove_lobby(&mut resync_match_lobby_events, *event.component.id);
    }
}
