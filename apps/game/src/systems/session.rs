use bevy_ecs::{event::EventReader, system::{Query, Res, ResMut}, schedule::{NextState, State}};

use game_engine::{
    asset::{
        AssetLoadedEvent, AssetType
    },
    logging::info,
    ui::UiManager,
    session::{SessionInsertComponentEvent, components::GlobalChatMessage},
};

use crate::{ui::{on_ui_load, UiCatalog}, states::AppState, resources::global_chat::GlobalChat};

pub fn session_load_asset_events(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ui_manager: ResMut<UiManager>,
    mut ui_catalog: ResMut<UiCatalog>,
    mut global_chat_messages: ResMut<GlobalChat>,
    message_q: Query<&GlobalChatMessage>,
    mut event_reader: EventReader<AssetLoadedEvent>,
) {
    for event in event_reader.read() {
        let asset_id = event.asset_id;
        let asset_type = event.asset_type;
        info!("received Asset Loaded Event! (asset_id: {:?})", asset_id);
        if asset_type == AssetType::Ui {
            let state = *state.get();
            on_ui_load(state, &mut next_state, &mut ui_manager, &mut ui_catalog, &mut global_chat_messages, &message_q, asset_id);
        }
    }
}

pub fn recv_inserted_global_chat_component(
    mut ui_manager: ResMut<UiManager>,
    mut global_chat_messages: ResMut<GlobalChat>,
    mut event_reader: EventReader<SessionInsertComponentEvent<GlobalChatMessage>>,
    chat_q: Query<&GlobalChatMessage>,
) {
    for event in event_reader.read() {
        // info!("received Inserted GlobalChatMessage from Session Server! (entity: {:?})", event.entity);

        if let Ok(chat) = chat_q.get(event.entity) {

            let chat_id = *chat.id;

            // let user_id = *chat.user_id;
            // let timestamp = *chat.timestamp;
            // let message = &*chat.message;
            // info!("incoming global message: [ user_id({:?}) | {:?} | {:?} | {:?} ]", user_id, timestamp, event.entity, message);

            global_chat_messages.add_message(&mut ui_manager, &chat_q, chat_id, event.entity);
        }
    }
}
