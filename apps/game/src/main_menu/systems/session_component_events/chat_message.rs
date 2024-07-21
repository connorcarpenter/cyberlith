use bevy_app::{App, Plugin, Update};
use bevy_ecs::{
    change_detection::ResMut,
    event::{EventReader, EventWriter},
    prelude::Query,
};

use game_engine::{
    logging::info,
    session::{
        components::{ChatMessage, ChatMessageGlobal, ChatMessageLocal, Lobby},
        SessionClient, SessionInsertComponentEvent,
    },
};

use crate::main_menu::{ui::events::ResyncMessageListUiEvent, resources::{chat_message_manager::ChatMessageManager, chat_message_events::ChatMessageEvents}};

pub struct ChatMessageComponentEventsPlugin;

impl Plugin for ChatMessageComponentEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, recv_inserted_chat_message_components)
            // updated_chat_message_component?
            // removed_chat_message_component?
        ;
    }
}

fn recv_inserted_chat_message_components(
    session_client: SessionClient,
    mut chat_message_manager: ResMut<ChatMessageManager>,
    mut chat_message_events: ResMut<ChatMessageEvents>,
    mut resync_message_list_ui_events: EventWriter<ResyncMessageListUiEvent>,
    mut chat_message_event_reader: EventReader<SessionInsertComponentEvent<ChatMessage>>,
    mut chat_message_global_event_reader: EventReader<
        SessionInsertComponentEvent<ChatMessageGlobal>,
    >,
    mut chat_message_local_event_reader: EventReader<SessionInsertComponentEvent<ChatMessageLocal>>,
    chat_message_q: Query<&ChatMessage>,
    local_chat_message_q: Query<&ChatMessageLocal>,
    lobby_q: Query<&Lobby>,
) {
    for (message_entity, is_global) in chat_message_events.recv_inserted_component_events(
        &mut chat_message_event_reader,
        &mut chat_message_global_event_reader,
        &mut chat_message_local_event_reader,
    ) {
        let message = chat_message_q.get(message_entity).unwrap();
        let message_id = *message.id;

        let timestamp = *message.timestamp;
        let message = &*message.message;

        let lobby_id_opt = match is_global {
            true => None,
            false => {
                let local_message = local_chat_message_q.get(message_entity).unwrap();
                let lobby_entity = local_message.lobby_entity.get(&session_client).unwrap();
                let lobby = lobby_q.get(lobby_entity).unwrap();
                Some(*lobby.id)
            }
        };

        if lobby_id_opt.is_none() {
            info!(
            "received Inserted Global ChatMessage from Session Server!  [ {:?} | {:?} | {:?} ]",
            timestamp, message_entity, message
        );
        } else {
            info!(
            "received Inserted Lobby({:?}) ChatMessage from Session Server!  [ {:?} | {:?} | {:?} ]",
                 lobby_id_opt.unwrap(), timestamp, message_entity, message,
            );
        }

        chat_message_manager.recv_message(
            &lobby_id_opt,
            &mut resync_message_list_ui_events,
            message_id,
            message_entity,
        );
    }
}
