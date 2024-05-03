use game_engine::{
    asset::{AssetId, ETag},
    render::base::Color,
};
use ui_builder::{Alignment, UiConfig, UiConfigBuild};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "start";
    let ui_asset_id_str = "tpp7za"; // AssetId::get_random(); // keep this around to generate new AssetIds if needed!
    let icon_asset_id_str = "34mvvk";
    let ui_etag = ETag::gen_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(ui_asset_id_str).unwrap();
    let icon_asset_id = AssetId::from_str(icon_asset_id_str).unwrap();

    // Create UI !
    let mut ui_config = UiConfig::new();

    // styles
    let window_style = ui_config.create_panel_style(|s| {
        s
            //.set_background_color(Color::BLACK)
            .set_background_alpha(0.0)
            .set_padding_px(10.0, 10.0, 10.0, 10.0)
            .set_vertical()
            .set_row_between_px(10.0);
    });
    let container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.0)
            .set_size_pc(100., 38.)
            .set_solid_fit()
            .set_aspect_ratio(16., 4.);
    });
    let text_style = ui_config.create_text_style(|s| {
        s.set_size_pc(80.);
    });
    // let base_button_style = ui_config.create_button_style(|s| {
    //     s.set_background_color(Color::DARK_GRAY)
    //         .set_hover_color(Color::RED)
    //         .set_down_color(Color::BLUE)
    //         .set_self_halign(Alignment::Center)
    //         .set_size_pc(50.0, 20.0)
    //         .set_size_max_px(240.0, 90.0)
    //         .set_solid_fit()
    //         .set_aspect_ratio(16.0, 4.)
    //         .set_padding_px(10.0, 10.0, 10.0, 10.0);
    // });
    let login_button_style = ui_config.create_button_style(|s| {
        s
            // .set_parent_style(base_button_style)
            .set_background_color(Color::DARK_GRAY)
            .set_hover_color(Color::RED)
            .set_down_color(Color::BLUE)
            .set_self_halign(Alignment::Center)
            .set_size_pc(50.0, 20.0)
            .set_size_max_px(240.0, 90.0)
            .set_solid_fit()
            .set_aspect_ratio(16.0, 4.)
            .set_padding_px(10.0, 10.0, 10.0, 10.0)
            .set_margin_right_vp(4.0);
    });
    let register_button_style = ui_config.create_button_style(|s| {
        s
            // .set_parent_style(base_button_style)
            .set_background_color(Color::DARK_GRAY)
            .set_hover_color(Color::RED)
            .set_down_color(Color::BLUE)
            .set_self_halign(Alignment::Center)
            .set_size_pc(50.0, 20.0)
            .set_size_max_px(240.0, 90.0)
            .set_solid_fit()
            .set_aspect_ratio(16.0, 4.)
            .set_padding_px(10.0, 10.0, 10.0, 10.0)
            .set_margin_left_vp(4.0);
    });

    // nodes

    ui_config
        .set_text_icon_asset_id(&icon_asset_id)
        .set_text_color(Color::WHITE)
        .root_mut()
        .set_style(window_style)
        .contents(|c| {
            // title container
            c.add_panel().set_style(container_style).contents(|c| {
                c.add_text("c y b e r l i t h").set_style(text_style);
            });

            // login button
            c.add_button("login_button")
                .set_as_first_input()
                .set_style(login_button_style)
                .contents(|c| {
                    c.add_text("login").set_style(text_style);
                })
                .navigation(|n| {
                    n.down_goes_to("register_button")
                        .right_goes_to("register_button");
                });

            // register button
            c.add_button("register_button")
                .set_style(register_button_style)
                .contents(|c| {
                    c.add_text("register").set_style(text_style);
                })
                .navigation(|n| {
                    n.up_goes_to("login_button").left_goes_to("login_button");
                });
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
