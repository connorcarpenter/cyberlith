use bevy_app::{App, Plugin, Update};
use bevy_ecs::{
    change_detection::{Res, ResMut},
    event::{EventReader, EventWriter},
    prelude::Query,
};

use game_engine::{
    logging::info,
    session::{components::ChatMessage, SessionInsertComponentEvent},
};

use crate::{
    resources::{chat_message_manager::ChatMessageManager, lobby_manager::LobbyManager},
    ui::events::ResyncChatMessageUiEvent,
};

pub struct ChatMessageComponentEventsPlugin;

impl Plugin for ChatMessageComponentEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, recv_inserted_chat_message_component)
            // updated_chat_message_component?
            // removed_chat_message_component?

        // TODO: ChatMessageGlobal component?
        // TODO: ChatMessageLocal component?
        ;

    }
}

fn recv_inserted_chat_message_component(
    lobby_manager: Res<LobbyManager>,
    mut message_manager: ResMut<ChatMessageManager>,
    mut resync_message_ui_events: EventWriter<ResyncChatMessageUiEvent>,
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
            "received Inserted ChatMessage from Session Server!  [ {:?} | {:?} | {:?} ]",
            timestamp, event.entity, message
        );

        message_manager.recv_message(
            &None,
            &mut resync_message_ui_events,
            message_id,
            event.entity,
        );
    }
}
