use bevy_ecs::{
    event::EventReader,
    schedule::{NextState, State},
    system::{Query, Res, ResMut},
};

use game_engine::{
    asset::{AssetLoadedEvent, AssetManager, AssetType},
    logging::info,
    session::{components::{GlobalChatMessage, PresentUserInfo}, SessionInsertComponentEvent},
    ui::UiManager,
};

use crate::{
    resources::{user_presence::UserPresence, global_chat::GlobalChat, on_asset_load, AssetCatalog},
    states::AppState,
    ui::{on_ui_load, UiCatalog},
};

pub fn session_load_asset_events(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut ui_manager: ResMut<UiManager>,
    mut ui_catalog: ResMut<UiCatalog>,
    asset_manager: Res<AssetManager>,
    mut asset_catalog: ResMut<AssetCatalog>,
    user_presence: Res<UserPresence>,
    mut global_chat_messages: ResMut<GlobalChat>,
    user_q: Query<&PresentUserInfo>,
    message_q: Query<&GlobalChatMessage>,
    mut event_reader: EventReader<AssetLoadedEvent>,
) {
    for event in event_reader.read() {
        let asset_id = event.asset_id;
        let asset_type = event.asset_type;
        match asset_type {
            AssetType::Ui => {
                info!("received Asset Loaded Ui Event! (asset_id: {:?})", asset_id);
                let state = *state.get();
                on_ui_load(
                    state,
                    &mut next_state,
                    &mut ui_manager,
                    &mut ui_catalog,
                    &asset_manager,
                    &user_presence,
                    &mut global_chat_messages,
                    &user_q,
                    &message_q,
                    asset_id,
                );
            }
            _ => {
                info!(
                    "received Asset Loaded Icon Event! (asset_id: {:?})",
                    asset_id
                );
                on_asset_load(&mut ui_manager, &mut asset_catalog, asset_id);
            }
        }
    }
}

pub fn recv_inserted_global_chat_component(
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut global_chat_messages: ResMut<GlobalChat>,
    user_presence: Res<UserPresence>,
    mut event_reader: EventReader<SessionInsertComponentEvent<GlobalChatMessage>>,
    user_q: Query<&PresentUserInfo>,
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

            global_chat_messages.recv_message(
                &mut ui_manager,
                &asset_manager,
                &user_presence,
                &user_q,
                &chat_q,
                chat_id,
                event.entity,
            );
        }
    }
}

pub fn recv_inserted_present_user_component(
    mut user_presence: ResMut<UserPresence>,
    mut event_reader: EventReader<SessionInsertComponentEvent<PresentUserInfo>>,
    users_q: Query<&PresentUserInfo>,
) {
    for event in event_reader.read() {
        info!("received Inserted PresentUserInfo from Session Server! (entity: {:?})", event.entity);

        if let Ok(user_info) = users_q.get(event.entity) {
            let user_id = *user_info.id;
            let user_name = &*user_info.name;

            info!("incoming user: [ user_id({:?}), entity({:?}), name({:?}) ]", user_id, event.entity, user_name);

            user_presence.recv_user(
                user_id,
                event.entity,
            );
        }
    }
}
