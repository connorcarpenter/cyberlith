use std::collections::BTreeMap;

use game_engine::{ui::UiManager, asset::AssetManager};

use crate::app::global::Global;

pub(crate) fn setup_global_chat_test_case(
    global: &mut Global,
    ui_manager: &mut UiManager,
    asset_manager: &AssetManager,
) {
    // main menu ui
    let main_menu_ui_handle = global.ui_handles[0];

    // global chat sub-ui
    let global_chat_ui_handle = global.ui_handles[1];

    // setup sub ui
    ui_manager.set_ui_container_contents(
        &main_menu_ui_handle,
        "center_container",
        &global_chat_ui_handle,
    );

    // setup global chat list
    global
        .global_chat_list_ui_ext
        .set_container_ui(ui_manager, &global_chat_ui_handle, "chat_wall");

    // setup chats
    global.global_chats = setup_global_chats();

    global.sync_chat_collections(ui_manager, asset_manager);

    // setup user list
    global
        .user_list_ui_ext
        .set_container_ui(ui_manager, &main_menu_ui_handle, "user_list");

    // setup users
    global.users = setup_users();

    global.sync_user_collections(ui_manager, asset_manager);
}

fn setup_users() -> BTreeMap<u32, String> {
    let mut users = Vec::new();
    users.push("tom");
    users.push("ben");
    users.push("andrew");
    users.push("joe");
    users.push("jane");
    users.push("sarah");
    users.push("jim");
    users.push("bob");

    let mut user_map = BTreeMap::<u32, String>::new();

    for user in users {
        user_map.insert(user_map.len() as u32, user.to_string());
    }

    user_map
}

fn setup_global_chats() -> BTreeMap<u32, (String, u8, u8, u8, u8, String)> {
    let mut users = Vec::new();
    users.push("tom");
    users.push("ben");
    users.push("andrew");
    users.push("joe");
    users.push("jane");
    users.push("sarah");
    users.push("jim");
    users.push("bob");

    let mut messages = Vec::new();
    messages.push("hello");
    messages.push("woah");
    messages.push("jeesh");
    messages.push("mmkay");
    messages.push("huh");
    messages.push("what");
    messages.push("ok");
    messages.push("sure");
    messages.push("nope");
    messages.push("yep");
    messages.push("maybe");
    messages.push("never");
    messages.push("always");
    messages.push("sometimes");
    messages.push("often");
    messages.push("rarely");
    messages.push("blah");
    messages.push("meh");

    let mut global_chats = BTreeMap::<u32, (String, u8, u8, u8, u8, String)>::new();

    let mut current_time = (3, 1, 11, 30);
    let mut current_user_index = 0;

    for _i in 0..64 {
        if random::gen_range_u32(0, 5) < 1 {
            current_user_index = random::gen_range_u32(0, users.len() as u32) as usize;
        }
        let message_index = random::gen_range_u32(0, messages.len() as u32) as usize;
        setup_global_chat(
            &mut global_chats,
            &mut current_time,
            users[current_user_index],
            messages[message_index],
        );
    }

    global_chats
}

fn setup_global_chat(
    global_chats: &mut BTreeMap<u32, (String, u8, u8, u8, u8, String)>,
    current_time: &mut (u32, u32, u32, u32),
    username: &str,
    message: &str,
) {
    let id = global_chats.len() as u32;
    let message = format!("{:?} {:?}", id, message);

    let (month, day, hour, minute) = current_time;

    global_chats.insert(
        global_chats.len() as u32,
        (
            username.to_string(),
            *month as u8,
            *day as u8,
            *hour as u8,
            *minute as u8,
            message,
        ),
    );

    let add_minutes = random::gen_range_u32(1, 300); // 1 minutes to 1/2 day
    *minute += add_minutes;
    while *minute >= 60 {
        *minute -= 60;
        *hour += 1;
    }
    while *hour >= 24 {
        *hour -= 24;
        *day += 1;
    }
    while *day >= 31 {
        *day -= 31;
        *month += 1;
    }
}