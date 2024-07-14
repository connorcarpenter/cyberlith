use bevy_app::{App, Plugin, Update};
use bevy_ecs::{prelude::Query, event::{EventReader, EventWriter}, change_detection::ResMut};

use game_engine::{logging::info, session::{SessionInsertComponentEvent, SessionRemoveComponentEvent, components::Lobby}};

use crate::{ui::events::ResyncLobbyUiEvent, resources::lobby_manager::LobbyManager};

pub struct LobbyComponentEventsPlugin;

impl Plugin for LobbyComponentEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, recv_inserted_lobby_component)
            .add_systems(Update, recv_removed_lobby_component);
    }
}

fn recv_inserted_lobby_component(
    mut lobby_manager: ResMut<LobbyManager>,
    mut resync_lobby_ui_events: EventWriter<ResyncLobbyUiEvent>,
    lobby_q: Query<&Lobby>,
    mut insert_lobby_component_event_reader: EventReader<SessionInsertComponentEvent<Lobby>>,
) {
    for event in insert_lobby_component_event_reader.read() {
        let lobby = lobby_q.get(event.entity).unwrap();
        let lobby_id = *lobby.id;

        let lobby_name = &*lobby.name;
        info!(
            "incoming lobby: [ {:?} | {:?} ]",
            event.entity, lobby_name
        );

        lobby_manager.recv_lobby(lobby_id, event.entity, &mut resync_lobby_ui_events);
    }
}

fn recv_removed_lobby_component(
    mut lobby_manager: ResMut<LobbyManager>,
    mut resync_lobby_ui_events: EventWriter<ResyncLobbyUiEvent>,
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