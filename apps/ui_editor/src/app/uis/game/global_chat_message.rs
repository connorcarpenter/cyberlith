use game_engine::{
    asset::{AssetId, ETag},
    render::base::Color,
};
use ui_builder::{Alignment, UiConfig, UiConfigBuild};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "global_chat_message";
    let ui_asset_id_str = "cxc6zk"; // AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
    let ui_etag = ETag::gen_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(&ui_asset_id_str).unwrap();

    // Create UI !
    let mut ui_config = UiConfig::new();

    // styles
    let container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            // .set_background_color(Color::GREEN)
            .set_horizontal()
            .set_self_halign(Alignment::Start)
            .set_margin_left_vp(2.0);
    });
    let message_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            // .set_background_color(Color::GREEN)
            .set_size_px(24.0)
            .set_text_color(Color::LIGHT_GRAY)
            .set_self_halign(Alignment::Start);
    });

    // nodes
    ui_config
        .root_mut()
        .set_style(container_style)
        .contents(|c| {
            // message
            c.add_text_with_id("yode yode yode dubags", "message")
                .set_style(message_style);
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
