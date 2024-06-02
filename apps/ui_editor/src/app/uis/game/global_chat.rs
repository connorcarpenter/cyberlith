use game_engine::{
    asset::{AssetId, ETag},
    render::base::Color,
};
use ui_builder::{Alignment, UiConfig, UiConfigBuild};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "global_chat";
    let ui_asset_id_str = "ngffab"; //AssetId::get_random().as_string(); // keep this around to generate new AssetIds if needed!
    let text_icon_asset_id_str = "34mvvk"; // this probably shouldn't change, it's the text font
    let eye_icon_asset_id_str = "qbgz5j"; // this probably shouldn't change, it's the password eye
    let ui_etag = ETag::gen_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(&ui_asset_id_str).unwrap();
    let text_icon_asset_id = AssetId::from_str(text_icon_asset_id_str).unwrap();
    let eye_icon_asset_id = AssetId::from_str(eye_icon_asset_id_str).unwrap();

    // Create UI !
    let mut ui_config = UiConfig::new();

    // styles
    let window_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_vertical()
            .set_children_valign(Alignment::Start);
    });
    let chat_wall_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_width_pc(100.0)
            .set_height_pc(95.0)
            .set_vertical()
            .set_children_valign(Alignment::End);
    });
    let base_textbox_style = ui_config.create_textbox_style(|s| {
        s.set_background_color(Color::GRAY)
            .set_hover_color(Color::RED)
            .set_active_color(Color::BLUE)
            .set_selection_color(Color::DARK_BLUE)
            .set_size_pc(100., 4.);
    });

    // nodes
    ui_config
        .root_mut()
        .set_style(window_style)
        .contents(|c| {
            // chat wall
            c.add_panel_with_id("chat_wall")
                .set_style(chat_wall_style);

            // message input
            // text-edit
            c.add_textbox("message_textbox")
                .set_as_first_input()
                .set_style(base_textbox_style);
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
