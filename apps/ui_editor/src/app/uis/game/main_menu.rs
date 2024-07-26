use game_engine::{
    asset::{AssetId, ETag},
    render::base::Color,
};
use ui_builder::{Alignment, UiConfig, UiConfigBuild};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "main_menu";
    let ui_asset_id_str = "kmqkp9"; //AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
    let ui_etag = ETag::gen_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(&ui_asset_id_str).unwrap();

    // Create UI !
    let mut ui_config = UiConfig::new();

    // styles
    let window_style = ui_config.create_panel_style(|s| {
        s
            //.set_background_color(Color::BLACK)
            .set_background_alpha(0.0)
            .set_horizontal();
    });

    let left_bar_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.0)
            .set_vertical()
            .set_children_valign(Alignment::Start)
            .set_width_pc(15.0)
            .set_height_pc(100.0);
    });
    let right_bar_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.0)
            .set_vertical()
            .set_children_valign(Alignment::Start)
            .set_width_pc(10.0)
            .set_height_pc(100.0);
    });
    let center_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.0)
            .set_vertical()
            .set_children_valign(Alignment::Start)
            .set_width_pc(75.0)
            .set_height_pc(100.0);
    });

    let left_top_bar_style = ui_config.create_panel_style(|s| {
        s.set_background_color(Color::DARK_GRAY)
            .set_width_pc(100.0)
            .set_height_vp(5.0)
            .set_horizontal()
            .set_children_halign(Alignment::Start);
    });
    let right_top_bar_style = ui_config.create_panel_style(|s| {
        s.set_background_color(Color::DARK_GRAY)
            .set_width_pc(100.0)
            .set_height_vp(5.0);
    });
    let center_top_bar_style = ui_config.create_panel_style(|s| {
        s.set_background_color(Color::DARK_GRAY)
            .set_width_pc(100.0)
            .set_height_vp(5.0);
    });

    let center_container_style = ui_config.create_ui_container_style(|s| {
        s.set_width_pc(100.0).set_height_vp(95.0);
    });
    let right_user_list_style = ui_config.create_panel_style(|s| {
        s.set_background_color(Color::DARK_GRAY)
            .set_width_pc(100.0)
            .set_height_vp(75.0)
            .set_children_valign(Alignment::Start);
    });

    let left_title_text_style = ui_config.create_text_style(|s| {
        s.set_size_pc(100.0).set_margin_left_pc(8.0);
    });
    let center_title_text_style = ui_config.create_text_style(|s| {
        s.set_size_pc(100.);
    });
    let base_button_style = ui_config.create_button_style(|s| {
        s.set_background_color(Color::DARK_GRAY)
            .set_hover_color(Color::GRAY)
            .set_down_color(Color::LIGHT_GRAY)
            .set_disabled_color(Color::LIGHT_GRAY);
    });
    let side_button_style = ui_config.create_button_style(|s| {
        s.set_parent_style(base_button_style)
            .set_width_pc(100.0)
            .set_height_vp(4.0)
            .set_margin_top_vp(0.2);
    });
    let side_space_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.0)
            .set_width_pc(100.0)
            .set_height_vp(4.0)
            .set_margin_top_vp(0.2);
    });
    let base_button_text_style = ui_config.create_text_style(|s| {
        s.set_size_pc(100.0)
            .set_self_halign(Alignment::Center)
            .set_text_color(Color::WHITE);
    });

    // nodes

    ui_config.root_mut().set_style(window_style).contents(|c| {
        // left bar
        c.add_panel().set_style(left_bar_style).contents(|c| {
            // top-most
            c.add_panel().set_style(left_top_bar_style).contents(|c| {
                c.add_text("c y b e r l i t h")
                    .set_style(left_title_text_style);
            });

            // left buttons

            // host match
            c.add_button("host_match_button")
                .set_style(side_button_style)
                .contents(|c| {
                    c.add_text("host match").set_style(base_button_text_style);
                });

            // join match
            c.add_button("join_match_button")
                .set_style(side_button_style)
                .contents(|c| {
                    c.add_text("join match").set_style(base_button_text_style);
                });

            // chat
            c.add_button("chat_button")
                .set_style(side_button_style)
                .contents(|c| {
                    c.add_text("chat").set_style(base_button_text_style);
                });

            // devlog
            c.add_button("devlog_button")
                .set_enabled(false)
                .set_style(side_button_style)
                .contents(|c| {
                    c.add_text("devlog").set_style(base_button_text_style);
                });

            // settings
            c.add_button("settings_button")
                .set_enabled(false)
                .set_style(side_button_style)
                .contents(|c| {
                    c.add_text("settings").set_style(base_button_text_style);
                });

            // blank space
            c.add_panel().set_style(side_space_style).contents(|c| {});

            // current lobby
            c.add_button("current_lobby_button")
                .set_visible(false)
                .set_enabled(false)
                .set_style(side_button_style)
                .contents(|c| {
                    c.add_text_with_id("-", "current_lobby_button_text")
                        .set_style(base_button_text_style);
                });
        });

        // center
        c.add_panel().set_style(center_style).contents(|c| {
            // center title
            c.add_panel().set_style(center_top_bar_style).contents(|c| {
                c.add_text_with_id("-", "center_title_text")
                    .set_style(center_title_text_style);
            });

            // main
            c.add_ui_container("center_container")
                .set_style(center_container_style);
        });

        // right bar
        c.add_panel().set_style(right_bar_style).contents(|c| {
            // top-most
            c.add_panel().set_style(right_top_bar_style).contents(|c| {
                // c.add_button("top_right_button")
                //     .set_as_first_input()
                //     .set_style(top_right_x_button_style)
                //     .contents(|c| {
                //         c.add_text("x").set_style(base_button_text_style);
                //     });
            });

            // user list
            c.add_panel_with_id("user_list")
                .set_style(right_user_list_style);

            // start button
            c.add_button("start_button")
                .set_visible(false)
                .set_style(side_button_style)
                .contents(|c| {
                    c.add_text("start").set_style(base_button_text_style);
                });

            // leave button
            c.add_button("leave_button")
                .set_visible(false)
                .set_style(side_button_style)
                .contents(|c| {
                    c.add_text("leave").set_style(base_button_text_style);
                });
        });
    });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
