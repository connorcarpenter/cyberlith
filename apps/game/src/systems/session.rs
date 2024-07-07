use bevy_ecs::{
    event::EventReader,
    schedule::{NextState, State},
    system::{Query, Res, ResMut},
};

use game_engine::{
    asset::{AssetLoadedEvent, AssetManager, AssetType},
    logging::info,
    session::{SessionClient, SessionUpdateComponentEvent, SessionRemoveComponentEvent, components::{GlobalChatMessage, PublicUserInfo}, SessionInsertComponentEvent},
    ui::UiManager,
};

use crate::{
    resources::{user_manager::UserManager, global_chat::GlobalChat, on_asset_load, AssetCatalog},
    states::AppState,
    ui::{on_ui_load, UiCatalog},
};

pub fn session_load_asset_events(
    state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    session_client: SessionClient,
    mut ui_manager: ResMut<UiManager>,
    mut ui_catalog: ResMut<UiCatalog>,
    asset_manager: Res<AssetManager>,
    mut user_manager: ResMut<UserManager>,
    mut asset_catalog: ResMut<AssetCatalog>,
    mut global_chat_messages: ResMut<GlobalChat>,
    user_q: Query<&PublicUserInfo>,
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
                    &session_client,
                    &mut ui_manager,
                    &mut ui_catalog,
                    &asset_manager,
                    &mut user_manager,
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
    session_client: SessionClient,
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut global_chat_messages: ResMut<GlobalChat>,
    mut event_reader: EventReader<SessionInsertComponentEvent<GlobalChatMessage>>,
    user_q: Query<&PublicUserInfo>,
    chat_q: Query<&GlobalChatMessage>,
) {
    for event in event_reader.read() {
        // info!("received Inserted GlobalChatMessage from Session Server! (entity: {:?})", event.entity);

        let chat = chat_q.get(event.entity).unwrap();
        let chat_id = *chat.id;

        // let user_id = *chat.user_id;
        // let timestamp = *chat.timestamp;
        // let message = &*chat.message;
        // info!("incoming global message: [ user_id({:?}) | {:?} | {:?} | {:?} ]", user_id, timestamp, event.entity, message);

        global_chat_messages.recv_message(
            &session_client,
            &mut ui_manager,
            &asset_manager,
            &user_q,
            &chat_q,
            chat_id,
            event.entity,
        );
    }
}

pub fn recv_inserted_public_user_info_component(
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut user_manager: ResMut<UserManager>,
    mut event_reader: EventReader<SessionInsertComponentEvent<PublicUserInfo>>,
    users_q: Query<&PublicUserInfo>,
) {
    for event in event_reader.read() {
        info!("received Inserted PublicUserInfo from Session Server! (entity: {:?})", event.entity);

        // let user_info = users_q.get(event.entity).unwrap();
        // let user_name = &*user_info.name;
        //
        // info!("incoming user: [ entity({:?}), name({:?}) ]", event.entity, user_name);

        user_manager.insert_user(
            &mut ui_manager,
            &asset_manager,
            &users_q,
            event.entity,
        );
    }
}

pub fn recv_updated_public_user_info_component(
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut user_manager: ResMut<UserManager>,
    mut event_reader: EventReader<SessionUpdateComponentEvent<PublicUserInfo>>,
    users_q: Query<&PublicUserInfo>,
) {
    for event in event_reader.read() {
        info!("received Updated PublicUserInfo from Session Server! (entity: {:?})", event.entity);

        user_manager.update_user(
            &mut ui_manager,
            &asset_manager,
            &users_q,
        );
    }
}

pub fn recv_removed_public_user_info_component(
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut user_manager: ResMut<UserManager>,
    mut event_reader: EventReader<SessionRemoveComponentEvent<PublicUserInfo>>,
    users_q: Query<&PublicUserInfo>,
) {
    for event in event_reader.read() {
        info!("received Removed PublicUserInfo from Session Server! (entity: {:?})", event.entity);

        user_manager.delete_user(
            &mut ui_manager,
            &asset_manager,
            &users_q,
            &event.entity,
        );
    }
}