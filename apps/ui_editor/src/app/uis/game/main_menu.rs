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
        s
            //.set_background_color(Color::BLACK)
            .set_background_alpha(0.0)
            .set_horizontal();
    });

    let left_bar_style = ui_config.create_panel_style(|s| {
        s
            .set_background_color(Color::BLUE)
            .set_vertical()
            .set_children_valign(Alignment::Start)
            .set_width_pc(20.0)
            .set_height_pc(100.0);
    });
    let right_bar_style = ui_config.create_panel_style(|s| {
        s
            .set_background_color(Color::BLUE)
            .set_vertical()
            .set_children_valign(Alignment::Start)
            .set_width_pc(20.0)
            .set_height_pc(100.0);
    });
    let center_style = ui_config.create_panel_style(|s| {
        s
            .set_background_alpha(0.0)
            .set_background_color(Color::DARK_GREEN)
            .set_vertical()
            .set_children_valign(Alignment::Start)
            .set_width_pc(60.0)
            .set_height_pc(100.0);
    });

    let left_top_bar_style = ui_config.create_panel_style(|s| {
        s
            .set_background_color(Color::DARK_BLUE)
            .set_width_pc(100.0)
            .set_height_vp(5.0);
    });
    let right_top_bar_style = ui_config.create_panel_style(|s| {
        s
            .set_background_color(Color::DARK_BLUE)
            .set_width_pc(100.0)
            .set_height_vp(5.0);
    });
    let center_top_bar_style = ui_config.create_panel_style(|s| {
        s
            .set_background_color(Color::GREEN)
            .set_width_pc(100.0)
            .set_height_vp(5.0);
    });

    let title_text_style = ui_config.create_text_style(|s| {
        s.set_size_pc(100.);
    });
    let base_button_style = ui_config.create_button_style(|s| {
        s.set_background_color(Color::DARK_GRAY)
            .set_hover_color(Color::RED)
            .set_down_color(Color::BLUE);
    });
    let full_button_style = ui_config.create_button_style(|s| {
        s.set_parent_style(base_button_style)
            .set_size_pc(100.0, 100.0);
    });
    let base_button_text_style = ui_config.create_text_style(|s| {
        s.set_size_pc(100.0)
            .set_self_halign(Alignment::Center)
            .set_text_color(Color::WHITE);
    });

    // nodes

    ui_config
        .set_text_icon_asset_id(&text_icon_asset_id)
        .set_eye_icon_asset_id(&eye_icon_asset_id)
        .root_mut()
        .set_style(window_style)
        .contents(|c| {
            // left bar
            c
                .add_panel()
                .set_style(left_bar_style)
                .contents(|c| {
                    // top-most
                    c
                        .add_panel()
                        .set_style(left_top_bar_style)
                        .contents(|c| {

                        });
            });

            // center
            c
                .add_panel()
                .set_style(center_style)
                .contents(|c| {
                    // top-most
                    c
                        .add_panel()
                        .set_style(center_top_bar_style)
                        .contents(|c| {
                            c.add_text("c y b e r l i t h").set_style(title_text_style);
                        });
            });

            // right bar
            c
                .add_panel()
                .set_style(right_bar_style)
                .contents(|c| {
                    // top-most
                    c
                        .add_panel()
                        .set_style(right_top_bar_style)
                        .contents(|c| {
                            c
                                .add_button("top_right_button")
                                .set_as_first_input()
                                .set_style(full_button_style)
                                .contents(|c| {
                                    c.add_text("X").set_style(base_button_text_style);
                                });
                        });
            });
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
