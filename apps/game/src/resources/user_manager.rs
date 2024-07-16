use std::collections::{BTreeMap, HashMap};

use bevy_ecs::{entity::Entity, event::EventWriter, prelude::Query, system::Resource};

use game_engine::{
    asset::AssetManager,
    session::components::User,
    ui::{
        extensions::{ListUiExt, ListUiExtItem},
        UiHandle, UiManager,
    },
};

use crate::ui::{events::ResyncUserUiEvent, UiCatalog, UiKey};

#[derive(Resource)]
pub struct UserManager {
    next_id: u32,
    self_user_entity: Option<Entity>,
    users: BTreeMap<u32, Entity>,
    entity_to_user_id_map: HashMap<Entity, u32>,
    list_ui_ext: ListUiExt<u32>,
    item_ui: Option<UiHandle>,
}

impl Default for UserManager {
    fn default() -> Self {
        Self {
            next_id: 0,
            self_user_entity: None,
            users: BTreeMap::new(),
            entity_to_user_id_map: HashMap::new(),
            list_ui_ext: ListUiExt::new(true),
            item_ui: None,
        }
    }
}

impl UserManager {
    pub(crate) fn on_load_user_list_item_ui(
        &mut self,
        ui_catalog: &mut UiCatalog,
        resync_user_public_info_events: &mut EventWriter<ResyncUserUiEvent>,
    ) {
        let item_ui_key = UiKey::UserListItem;
        let item_ui_handle = ui_catalog.get_ui_handle(item_ui_key);

        ui_catalog.set_loaded(item_ui_key);

        self.item_ui = Some(item_ui_handle.clone());

        resync_user_public_info_events.send(ResyncUserUiEvent);
    }

    pub(crate) fn recv_main_menu_ui(
        &mut self,
        ui_manager: &mut UiManager,
        main_menu_ui_handle: &UiHandle,
    ) {
        self.list_ui_ext
            .set_container_ui(ui_manager, main_menu_ui_handle, "user_list");
    }

    pub fn set_self_user_entity(&mut self, resync_ui_events: &mut EventWriter<ResyncUserUiEvent>, user_entity: Entity) {
        if self.self_user_entity.is_some() {
            panic!("self_user_entity already set");
        }
        self.self_user_entity = Some(user_entity);

        resync_ui_events.send(ResyncUserUiEvent);
    }

    pub fn insert_user(
        &mut self,
        resync_events: &mut EventWriter<ResyncUserUiEvent>,
        user_entity: Entity,
    ) {
        let user_id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.users.insert(user_id, user_entity);
        self.entity_to_user_id_map.insert(user_entity, user_id);

        resync_events.send(ResyncUserUiEvent);
    }

    pub fn delete_user(
        &mut self,
        resync_events: &mut EventWriter<ResyncUserUiEvent>,
        user_entity: &Entity,
    ) {
        let user_id = self.entity_to_user_id_map.remove(user_entity).unwrap();
        self.users.remove(&user_id);

        resync_events.send(ResyncUserUiEvent);
    }

    pub fn sync_with_collection(
        &mut self,
        ui_manager: &mut UiManager,
        asset_manager: &AssetManager,
        user_q: &Query<&User>,
    ) {
        if self.item_ui.is_none() {
            return;
        }

        let item_ui_handle = self.item_ui.as_ref().unwrap();

        self.list_ui_ext.sync_with_collection(
            ui_manager,
            asset_manager,
            self.users.iter(),
            self.users.len(),
            |item_ctx, user_id, _| {
                let user_entity = self.users.get(&user_id).unwrap();
                let user_entity = *user_entity;
                if let Ok(user_public_info) = user_q.get(user_entity) {
                    let username = user_public_info.name.as_str();
                    let is_online = *user_public_info.online;
                    let is_self = {
                        if let Some(self_user_entity) = self.self_user_entity {
                            self_user_entity == user_entity
                        } else {
                            false
                        }
                    };
                    add_user_item(item_ctx, item_ui_handle, username, is_self, is_online);
                }
            },
        );
    }
}

fn add_user_item(item_ctx: &mut ListUiExtItem<u32>, ui: &UiHandle, username: &str, is_self: bool, is_online: bool) {
    item_ctx.add_copied_node(ui);
    item_ctx.set_text_by_id("username", username);
    if is_self {
        item_ctx.set_style_by_id("username", "self");
    } else {
        if is_online {
            item_ctx.set_style_by_id("username", "online");
        } else {
            item_ctx.set_style_by_id("username", "offline");
        }
    }
}
