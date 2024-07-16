use bevy_app::{App, Plugin, Update};
use bevy_ecs::{
    change_detection::ResMut,
    event::{EventReader, EventWriter},
    prelude::Query,
    system::Res,
};

use game_engine::{
    logging::{info, warn},
    session::{
        components::{Lobby, LobbyMember},
        SessionClient, SessionInsertComponentEvent, SessionRemoveComponentEvent,
    },
};

use crate::{
    resources::{lobby_manager::LobbyManager, user_manager::UserManager},
    ui::events::{
        ResyncLobbyListUiEvent, ResyncMainMenuUiEvent, ResyncMessageListUiEvent,
        ResyncUserListUiEvent,
    },
};

pub struct LobbyComponentEventsPlugin;

impl Plugin for LobbyComponentEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Lobby
            .add_systems(Update, recv_inserted_lobby_component)
            // updated_lobby_component?
            .add_systems(Update, recv_removed_lobby_component)
            // LobbyMember
            .add_systems(Update, recv_inserted_lobby_member_component)
            // updated_lobby_member_component?
            .add_systems(Update, recv_removed_lobby_member_component);
    }
}

fn recv_inserted_lobby_component(
    mut lobby_manager: ResMut<LobbyManager>,
    mut resync_lobby_ui_events: EventWriter<ResyncLobbyListUiEvent>,
    lobby_q: Query<&Lobby>,
    mut insert_lobby_component_event_reader: EventReader<SessionInsertComponentEvent<Lobby>>,
) {
    for event in insert_lobby_component_event_reader.read() {
        let lobby = lobby_q.get(event.entity).unwrap();
        let lobby_id = *lobby.id;

        let lobby_name = &*lobby.name;
        info!("incoming lobby: [ {:?} | {:?} ]", event.entity, lobby_name);

        lobby_manager.recv_lobby(lobby_id, event.entity, &mut resync_lobby_ui_events);
    }
}

fn recv_removed_lobby_component(
    mut lobby_manager: ResMut<LobbyManager>,
    mut resync_lobby_ui_events: EventWriter<ResyncLobbyListUiEvent>,
    mut remove_lobby_component_event_reader: EventReader<SessionRemoveComponentEvent<Lobby>>,
) {
    for event in remove_lobby_component_event_reader.read() {
        info!(
            "received Removed Lobby from Session Server! (entity: {:?})",
            event.entity
        );

        lobby_manager.remove_lobby(*event.component.id, &mut resync_lobby_ui_events);
    }
}

fn recv_inserted_lobby_member_component(
    session_client: SessionClient,
    user_manager: Res<UserManager>,
    mut lobby_manager: ResMut<LobbyManager>,
    lobby_q: Query<&Lobby>,
    lobby_member_q: Query<&LobbyMember>,
    mut resync_main_menu_ui_events: EventWriter<ResyncMainMenuUiEvent>,
    mut resync_chat_message_ui_events: EventWriter<ResyncMessageListUiEvent>,
    mut resync_user_ui_events: EventWriter<ResyncUserListUiEvent>,
    mut insert_lobby_member_event_reader: EventReader<SessionInsertComponentEvent<LobbyMember>>,
) {
    for event in insert_lobby_member_event_reader.read() {
        let lobby_member = lobby_member_q.get(event.entity).unwrap();
        let lobby_entity = lobby_member.lobby_entity.get(&session_client).unwrap();
        let lobby = lobby_q.get(lobby_entity).unwrap();
        let lobby_id = *lobby.id;
        let user_entity = lobby_member.user_entity.get(&session_client).unwrap();

        // insert user into lobby
        lobby_manager.put_user_in_lobby(user_entity, lobby_entity);

        let Some(self_user_entity) = user_manager.get_self_user_entity() else {
            warn!("self_user_entity not set when receiving inserted LobbyMember ..");
            continue;
        };

        if self_user_entity == user_entity {
            lobby_manager.set_current_lobby(
                &mut resync_main_menu_ui_events,
                &mut resync_chat_message_ui_events,
                &mut resync_user_ui_events,
                lobby_id,
            );
        }
    }
}

fn recv_removed_lobby_member_component(
    session_client: SessionClient,
    user_manager: Res<UserManager>,
    mut lobby_manager: ResMut<LobbyManager>,
    lobby_q: Query<&Lobby>,
    mut resync_main_menu_ui_events: EventWriter<ResyncMainMenuUiEvent>,
    mut resync_chat_message_ui_events: EventWriter<ResyncMessageListUiEvent>,
    mut resync_user_ui_events: EventWriter<ResyncUserListUiEvent>,
    mut remove_lobby_member_event_reader: EventReader<SessionRemoveComponentEvent<LobbyMember>>,
) {
    for event in remove_lobby_member_event_reader.read() {
        let lobby_member_entity = event.entity.clone();
        let lobby_member = event.component.clone();

        info!(
            "received Removed Lobby Member from Session Server! (entity: {:?})",
            lobby_member_entity
        );

        let lobby_entity = lobby_member.lobby_entity.get(&session_client).unwrap();
        let lobby = lobby_q.get(lobby_entity).unwrap();
        let lobby_id = *lobby.id;
        let user_entity = lobby_member.user_entity.get(&session_client).unwrap();

        // remove user from lobby
        lobby_manager.remove_user_from_lobby(&user_entity);

        let Some(self_user_entity) = user_manager.get_self_user_entity() else {
            warn!("self_user_entity not set when receiving removed LobbyMember ..");
            continue;
        };

        if user_entity == self_user_entity {
            let current_lobby_id = lobby_manager.get_current_lobby().unwrap();
            if current_lobby_id != lobby_id {
                panic!("current_lobby_id != lobby_id, when removing LobbyMember entity");
            }
            lobby_manager.leave_current_lobby(
                &mut resync_main_menu_ui_events,
                &mut resync_chat_message_ui_events,
                &mut resync_user_ui_events,
            );
        }
    }
}
