use game_engine::{
    asset::{AssetId, ETag},
    render::base::Color,
};
use ui_builder::{Alignment, UiConfig, UiConfigBuild};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "match_lobby_list_item";
    let ui_asset_id_str = "pup52m"; //AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
    let ui_etag = ETag::gen_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(&ui_asset_id_str).unwrap();

    // Create UI !
    let mut ui_config = UiConfig::new();

    // styles
    let container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_background_color(Color::YELLOW)
            .set_height_px(24.0)
            .set_width_pc(100.0);
    });
    let button_style = ui_config.create_button_style(|s| {
        s.set_background_alpha(1.)
            .set_background_color(Color::DARK_BLUE)
            .set_down_color(Color::RED)
            .set_hover_color(Color::BLUE)
            .set_size_pc(100.0, 100.0)
            .set_horizontal()
            .set_col_between_vp(2.0)
            .set_children_halign(Alignment::Start);
    });
    let match_name_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            // .set_background_color(Color::YELLOW)
            .set_size_pc(100.0)
            .set_margin_left_pc(2.0)
            .set_text_color(Color::LIGHT_GRAY)
            .set_self_halign(Alignment::Start);
    });
    let username_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            // .set_background_color(Color::AQUA)
            .set_size_pc(100.0)
            .set_text_color(Color::GRAY)
            .set_self_halign(Alignment::Start);
    });

    // nodes
    ui_config
        .root_mut()
        .set_style(container_style)
        .contents(|c| {
            c.add_button("lobby_button")
                .set_style(button_style)
                .contents(|c| {
                    // match name
                    c.add_text_with_id("my super cool match", "match_name")
                        .set_style(match_name_style);
                    // username
                    c.add_text_with_id("coolname", "username")
                        .set_style(username_style);
                });
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
