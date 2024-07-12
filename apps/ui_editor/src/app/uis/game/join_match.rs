use game_engine::{
    asset::{AssetId, ETag},
};

use ui_builder::{Alignment, UiConfig, UiConfigBuild};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "join_match";
    let ui_asset_id_str = "qqxe6s"; //AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
    let ui_etag = ETag::gen_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(&ui_asset_id_str).unwrap();

    // Create UI !
    let mut ui_config = UiConfig::new();

    // styles
    let window_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            // .set_background_color(Color::RED)
            .set_vertical()
            .set_children_valign(Alignment::Start);
    });
    let match_lobby_list_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            // .set_background_color(Color::BLUE)
            .set_width_pc(100.0)
            .set_height_pc(100.0)
            .set_vertical()
            .set_children_valign(Alignment::Start);
    });

    // nodes
    ui_config.root_mut().set_style(window_style).contents(|c| {
        // match lobby list
        c.add_panel_with_id("lobby_list").set_style(match_lobby_list_style);
    });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
