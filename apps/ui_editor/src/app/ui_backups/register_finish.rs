use game_engine::{
    asset::{AssetId, ETag},
    render::base::Color,
};
use ui_builder::{Alignment, UiConfig, UiConfigBuild};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "register_finish";
    let ui_asset_id_str = "fsfn5m"; // AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
    let icon_asset_id_str = "34mvvk"; // this probably shouldn't change, it's the text font
    let ui_etag = ETag::gen_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(&ui_asset_id_str).unwrap();
    let icon_asset_id = AssetId::from_str(icon_asset_id_str).unwrap();

    // Create UI !
    let mut ui_config = UiConfig::new();

    // styles
    let window_style = ui_config.create_panel_style(|s| {
        s
            .set_background_alpha(0.)
            .set_vertical()
            .set_children_halign(Alignment::Center);
    });
    let main_container_style = ui_config.create_panel_style(|s| {
        s
            .set_background_alpha(0.)
            .set_background_color(Color::DARK_GRAY)
            .set_size_pc(100., 100.)
            .set_solid_fit()
            .set_aspect_ratio(16., 9.)
            .set_self_halign(Alignment::Center);
    });
    let title_container_style = ui_config.create_panel_style(|s| {
        s
            .set_background_alpha(0.)
            // .set_background_color(Color::BLUE)
            .set_size_pc(100., 16.)
            .set_vertical()
            .set_children_valign(Alignment::Start);
    });
    let title_text_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.).set_size_pc(90.0);
    });
    let body_container_style = ui_config.create_panel_style(|s| {
        s
            .set_background_alpha(0.)
            // .set_background_color(Color::RED)
            .set_size_pc(100., 84.)
            .set_vertical()
            .set_children_valign(Alignment::Center)
            .set_row_between_vp(1.0);
    });
    let heading_text_line_container_style = ui_config.create_panel_style(|s| {
        s
            .set_background_alpha(0.)
            // .set_background_color(Color::YELLOW)
            .set_width_pc(50.0)
            .set_height_vp(6.0);
    });
    let heading_text_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            .set_size_vp(7.0)
            .set_self_halign(Alignment::Center);
    });
    let base_button_text_style = ui_config.create_text_style(|s| {
        s.set_size_vp(5.0)
            .set_self_halign(Alignment::Center)
            .set_self_valign(Alignment::Center)
            .set_margin_vp(1.0, 1.0, 1.0, 1.0);
    });
    let base_button_style = ui_config.create_button_style(|s| {
        s.set_background_color(Color::DARK_GRAY)
            .set_hover_color(Color::RED)
            .set_down_color(Color::BLUE);
    });
    let home_button_style = ui_config.create_button_style(|s| {
        s.set_parent_style(base_button_style)
            // .set_background_color(Color::DARK_GRAY)
            // .set_hover_color(Color::RED)
            // .set_down_color(Color::BLUE)
            .set_margin_top_vp(5.0)
            .set_self_halign(Alignment::Center);
    });

    // nodes
    ui_config
        .set_text_icon_asset_id(&icon_asset_id)
        .set_text_color(Color::WHITE)
        .root_mut()
        .set_style(window_style)
        .contents(|c| {
            // main container
            c.add_panel().set_style(main_container_style).contents(|c| {
                // title container
                c.add_panel()
                    .set_style(title_container_style)
                    .contents(|c| {
                        c.add_text("c y b e r l i t h").set_style(title_text_style);
                    });

                // body container
                c
                    .add_panel()
                    .set_style(body_container_style)
                    .contents(|c| {
                        // line 1
                        c
                            .add_panel()
                            .set_style(heading_text_line_container_style)
                            .contents(|c| {
                                c
                                    .add_text("thank you for registering!")
                                    .set_style(heading_text_style);
                            });

                        // line 2
                        c
                            .add_panel()
                            .set_style(heading_text_line_container_style)
                            .contents(|c| {
                                c
                                    .add_text("check your email for instructions")
                                    .set_style(heading_text_style);
                            });

                        // line 3
                        c
                            .add_panel()
                            .set_style(heading_text_line_container_style)
                            .contents(|c| {
                                c
                                    .add_text("to activate your account")
                                    .set_style(heading_text_style);
                            });

                        // home button
                        c
                            .add_button("submit_button")
                            .set_style(home_button_style)
                            .contents(|c| {
                                c
                                    .add_text("home").set_style(base_button_text_style);
                            })
                            .set_as_first_input();
                });
            });
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
