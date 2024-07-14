use bevy_app::{App, Plugin, Update};
use bevy_ecs::{prelude::Query, event::{EventReader, EventWriter}, change_detection::ResMut};

use game_engine::{session::{components::{LobbyPublic, MessagePublic, UserPublic}, SessionInsertComponentEvent, SessionRemoveComponentEvent, SessionUpdateComponentEvent}, logging::info};

use crate::{ui::events::{ResyncGlobalChatEvent, ResyncMatchLobbiesEvent, ResyncPublicUserInfoEvent}, resources::{user_manager::UserManager, match_lobbies::MatchLobbies, global_chat::GlobalChat}};

pub struct SessionComponentEventsPlugin;

impl Plugin for SessionComponentEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, recv_inserted_message_public_component)

            .add_systems(Update, recv_inserted_lobby_public_component)
            .add_systems(Update, recv_removed_lobby_public_component)

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
        info!("received Inserted PublicUserInfo from Session Server! (entity: {:?})", event.entity);

        // let user_info = users_q.get(event.entity).unwrap();
        // let user_name = &*user_info.name;
        //
        // info!("incoming user: [ entity({:?}), name({:?}) ]", event.entity, user_name);

        user_manager.insert_user(
            &mut resync_event_writer,
            event.entity,
        );
    }
}

pub fn recv_updated_user_public_info_component(
    mut update_user_public_event_reader: EventReader<SessionUpdateComponentEvent<UserPublic>>,
    mut resync_user_public_event_writer: EventWriter<ResyncPublicUserInfoEvent>,
) {
    for event in update_user_public_event_reader.read() {
        info!("received Updated PublicUserInfo from Session Server! (entity: {:?})", event.entity);

        resync_user_public_event_writer.send(ResyncPublicUserInfoEvent);
    }
}

pub fn recv_removed_user_public_component(
    mut user_manager: ResMut<UserManager>,
    mut resync_user_public_info_event_writer: EventWriter<ResyncPublicUserInfoEvent>,
    mut event_reader: EventReader<SessionRemoveComponentEvent<UserPublic>>,
) {
    for event in event_reader.read() {
        info!("received Removed PublicUserInfo from Session Server! (entity: {:?})", event.entity);

        user_manager.delete_user(
            &mut resync_user_public_info_event_writer,
            &event.entity,
        );
    }
}

pub fn recv_inserted_message_public_component(
    mut global_chat_messages: ResMut<GlobalChat>,
    mut resync_global_chat_events: EventWriter<ResyncGlobalChatEvent>,
    mut event_reader: EventReader<SessionInsertComponentEvent<MessagePublic>>,
    chat_q: Query<&MessagePublic>,
) {
    for event in event_reader.read() {
        // info!("received Inserted GlobalChatMessage from Session Server! (entity: {:?})", event.entity);

        let chat = chat_q.get(event.entity).unwrap();
        let chat_id = *chat.id;

        let timestamp = *chat.timestamp;
        let message = &*chat.message;
        info!("incoming global message: [ {:?} | {:?} | {:?} ]", timestamp, event.entity, message);

        global_chat_messages.recv_message(
            &mut resync_global_chat_events,
            chat_id,
            event.entity,
        );
    }
}

pub fn recv_inserted_lobby_public_component(
    mut match_lobbies: ResMut<MatchLobbies>,
    mut resync_match_lobby_events: EventWriter<ResyncMatchLobbiesEvent>,
    lobby_q: Query<&LobbyPublic>,
    mut insert_lobby_event_reader: EventReader<SessionInsertComponentEvent<LobbyPublic>>,
) {
    for event in insert_lobby_event_reader.read() {

        let lobby = lobby_q.get(event.entity).unwrap();
        let lobby_id = *lobby.id;

        let lobby_name = &*lobby.name;
        info!("incoming match lobby: [ {:?} | {:?} ]", event.entity, lobby_name);

        match_lobbies.recv_lobby(
            &mut resync_match_lobby_events,
            lobby_id,
            event.entity,
        );
    }
}

pub fn recv_removed_lobby_public_component(
    mut match_lobbies: ResMut<MatchLobbies>,
    mut resync_match_lobby_events: EventWriter<ResyncMatchLobbiesEvent>,
    mut remove_lobby_event_reader: EventReader<SessionRemoveComponentEvent<LobbyPublic>>,
) {
    for event in remove_lobby_event_reader.read() {
        info!("received Removed MatchLobby from Session Server! (entity: {:?})", event.entity);

        match_lobbies.remove_lobby(
            &mut resync_match_lobby_events,
            *event.component.id,
        );
    }
}