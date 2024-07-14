use game_engine::{
    asset::{AssetId, ETag},
    render::base::Color,
};
use ui_builder::{Alignment, UiConfig, UiConfigBuild};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "user_list_item";
    let ui_asset_id_str = "8ywqfp"; // AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
    let ui_etag = ETag::gen_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(&ui_asset_id_str).unwrap();

    // Create UI !
    let mut ui_config = UiConfig::new();

    // styles
    let container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_height_px(24.0)
            .set_horizontal()
            .set_self_halign(Alignment::Start)
            .set_margin_left_pc(2.0);
    });
    let username_style_online = ui_config.create_text_style(|s| {
        s.set_id("online")
            .set_background_alpha(0.)
            .set_size_pc(100.0)
            .set_text_color(Color::WHITE);
    });
    let username_style_offline = ui_config.create_text_style(|s| {
        s.set_id("offline")
            .set_background_alpha(0.)
            .set_size_pc(100.0)
            .set_text_color(Color::LIGHT_GRAY);
    });

    // nodes
    ui_config
        .root_mut()
        .set_style(container_style)
        .contents(|c| {
            // username
            c.add_text_with_id("?", "username")
                .set_style(username_style_offline);
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
