use game_engine::{
    asset::{AssetId, ETag},
    render::base::Color,
};
use ui_builder::{Alignment, UiConfig, UiConfigBuild};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "global_chat_2";
    let ui_asset_id_str = "n21q8k"; // AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
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
        s.set_background_alpha(1.)
            .set_background_color(Color::DARK_BLUE)
            .set_width_pc(100.0)
            .set_height_pc(95.0);
    });
    let base_textbox_style = ui_config.create_textbox_style(|s| {
        s.set_background_color(Color::GRAY)
            .set_hover_color(Color::RED)
            .set_active_color(Color::BLUE)
            .set_selection_color(Color::DARK_BLUE)
            .set_size_pc(100., 4.);
    });

    /////// global chat list style
    let list_container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_vertical()
            .set_children_valign(Alignment::End)
            .set_children_halign(Alignment::Start)
            .set_width_pc(100.0)
            .set_height_pc(100.0);
    });

    /////// global chat list item styles
    let item_container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(1.)
            .set_background_color(Color::DARK_GRAY)
            .set_horizontal()
            .set_height_vp(4.0)
            .set_self_halign(Alignment::Start)
            .set_self_valign(Alignment::Start)
            .set_children_valign(Alignment::Start)
            .set_children_halign(Alignment::Start);
    });
    let user_name_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            .set_size_vp(4.0)
            .set_margin_left_vp(2.0)
            .set_text_color(Color::WHITE);
    });
    let timestamp_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            .set_size_vp(2.0)
            .set_margin_left_vp(2.0)
            .set_text_color(Color::GRAY);
    });
    let message_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            // .set_background_color(Color::GREEN)
            .set_size_vp(3.0)
            .set_margin_left_vp(2.0)
            .set_text_color(Color::WHITE);
    });

    // nodes
    ui_config
        .set_text_icon_asset_id(&text_icon_asset_id)
        .set_eye_icon_asset_id(&eye_icon_asset_id)
        .root_mut()
        .set_style(window_style)
        .contents(|c| {
            // chat wall
            c.add_panel()
                .set_style(chat_wall_style)
                .contents(|c| {
                    // list container
                    c.add_panel()
                        .set_style(list_container_style)
                        .contents(|c| {
                            // item 1
                            c.add_panel().set_style(item_container_style).contents(|c| {
                                // username
                                c.add_text("sample name")
                                    .set_style(user_name_style);

                                // timestamp
                                c.add_text("4/2/3141 15:21 OM")
                                    .set_style(timestamp_style);

                                // message
                                c.add_text("hello world")
                                    .set_style(message_style);
                            });

                            // item 2
                            c.add_panel().set_style(item_container_style).contents(|c| {
                                // username
                                c.add_text("sample name")
                                    .set_style(user_name_style);

                                // timestamp
                                c.add_text("4/2/3141 15:21 OM")
                                    .set_style(timestamp_style);

                                // message
                                c.add_text("this is a test")
                                    .set_style(message_style);
                            });

                            // item 3
                            c.add_panel().set_style(item_container_style).contents(|c| {
                                // username
                                c.add_text("sample name")
                                    .set_style(user_name_style);

                                // timestamp
                                c.add_text("4/2/3141 15:21 OM")
                                    .set_style(timestamp_style);

                                // message
                                c.add_text("this is a test also")
                                    .set_style(message_style);
                            });

                            // item 4
                            c.add_panel().set_style(item_container_style).contents(|c| {
                                // username
                                c.add_text("sample name")
                                    .set_style(user_name_style);

                                // timestamp
                                c.add_text("4/2/3141 15:21 OM")
                                    .set_style(timestamp_style);

                                // message
                                c.add_text("okay")
                                    .set_style(message_style);
                            });

                            // item 5
                            c.add_panel().set_style(item_container_style).contents(|c| {
                                // username
                                c.add_text("sample name")
                                    .set_style(user_name_style);

                                // timestamp
                                c.add_text("4/2/3141 15:21 OM")
                                    .set_style(timestamp_style);

                                // message
                                c.add_text("goodbye")
                                    .set_style(message_style);
                            });
                        });
                });

            // message input
            // text-edit
            c.add_textbox("message_textbox")
                .set_as_first_input()
                .set_style(base_textbox_style);
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
