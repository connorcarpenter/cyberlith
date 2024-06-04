use game_engine::{
    asset::{AssetId, ETag},
    render::base::Color,
};
use ui_builder::{UiConfig, UiConfigBuild};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "global_chat_day_divider";
    let ui_asset_id_str = "3wnz6n"; // AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
    let ui_etag = ETag::gen_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(&ui_asset_id_str).unwrap();

    // Create UI !
    let mut ui_config = UiConfig::new();

    // styles
    let container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            // .set_background_color(Color::DARK_GRAY)
            .set_horizontal()
            .set_height_vp(4.0)
            .set_width_pc(97.0);
    });
    let divider_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(1.)
            .set_background_color(Color::DARK_GRAY)
            .set_height_px(1.0)
            .set_width_pc(45.0);
    });
    let timestamp_container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            // .set_background_color(Color::DARK_BLUE)
            .set_height_pc(100.0)
            .set_width_pc(10.0);
    });
    let timestamp_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            .set_size_px(20.0)
            .set_text_color(Color::GRAY);
    });

    // nodes
    ui_config
        .root_mut()
        .set_style(container_style)
        .contents(|c| {

            c.add_panel()
                .set_style(divider_style);

            c.add_panel()
                .set_style(timestamp_container_style)
                .contents(|c| {
                    // // timestamp
                    c.add_text_with_id("4/2/3141", "timestamp")
                        .set_style(timestamp_style);
                });

            c.add_panel()
                .set_style(divider_style);
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
