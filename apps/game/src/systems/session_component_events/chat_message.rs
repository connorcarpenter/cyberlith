use bevy_app::{App, Plugin, Update};
use bevy_ecs::{prelude::Query, event::{EventReader, EventWriter}, change_detection::{Res, ResMut}};

use game_engine::{logging::info, session::{components::ChatMessage, SessionInsertComponentEvent}};

use crate::{ui::events::ResyncLobbyGlobalEvent, resources::{lobby_manager::LobbyManager, chat_message_manager::ChatMessageManager}};

pub struct ChatMessageComponentEventsPlugin;

impl Plugin for ChatMessageComponentEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, recv_inserted_chat_message_component);
    }
}

fn recv_inserted_chat_message_component(
    lobby_manager: Res<LobbyManager>,
    mut message_manager: ResMut<ChatMessageManager>,
    mut resync_global_chat_events: EventWriter<ResyncLobbyGlobalEvent>,
    mut message_event_reader: EventReader<SessionInsertComponentEvent<ChatMessage>>,
    message_q: Query<&ChatMessage>,
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