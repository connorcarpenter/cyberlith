use game_engine::{
    asset::{AssetId, ETag},
    render::base::Color,
};
use ui_builder::{Alignment, UiConfig, UiConfigBuild};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "global_chat_username_and_message";
    let ui_asset_id_str = "ddbxab"; //AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
    let ui_etag = ETag::gen_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(&ui_asset_id_str).unwrap();

    // Create UI !
    let mut ui_config = UiConfig::new();

    // styles
    let container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(1.)
            .set_background_color(Color::RED)
            .set_vertical();
    });
    let top_container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(1.)
            .set_background_color(Color::BLUE)
            .set_horizontal()
            .set_height_vp(4.0);
    });
    let user_name_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            .set_size_px(30.0)
            .set_margin_left_vp(2.0)
            .set_text_color(Color::WHITE);
    });
    let timestamp_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            .set_size_px(18.0)
            .set_margin_left_vp(2.0)
            .set_text_color(Color::GRAY);
    });
    let message_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            // .set_background_color(Color::GREEN)
            .set_size_px(24.0)
            .set_margin_left_vp(2.0)
            .set_text_color(Color::WHITE);
    });

    // nodes
    ui_config
        .root_mut()
        .set_style(container_style)
        .contents(|c| {
            c.add_panel().set_style(top_container_style).contents(|c| {
                // username
                c.add_text_with_id("sample name", "user_name")
                    .set_style(user_name_style);

                // timestamp
                c.add_text_with_id("4/2/3141 15:21 OM", "timestamp")
                    .set_style(timestamp_style);
            });

            // message
            c.add_text_with_id("yode yode yode dubags", "message")
                .set_style(message_style);
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
