use game_engine::{ui::{Alignment, Ui}, render::base::Color, asset::{AssetId, ETag}};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, Ui) {
    // config
    let ui_name = "start";
    let ui_asset_id_str = "tpp7za"; // AssetId::get_random(); // keep this around to generate new AssetIds if needed!
    let icon_asset_id_str = "34mvvk";
    let ui_etag = ETag::new_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(ui_asset_id_str).unwrap();
    let icon_asset_id = AssetId::from_str(icon_asset_id_str).unwrap();

    // Create UI !
    let mut ui = Ui::new();

    // styles
    let window_style = ui.create_panel_style(|s| {
        s
            //.set_background_color(Color::BLACK)
            .set_background_alpha(0.0)
            .set_padding_px(10.0, 10.0, 10.0, 10.0)
            .set_vertical()
            .set_row_between_px(10.0);
    });
    let container_style = ui.create_panel_style(|s| {
        s.set_background_alpha(0.0)
            .set_size_pc(100., 38.)
            .set_solid_fit()
            .set_aspect_ratio(16., 4.);
    });
    let base_button_style = ui.create_button_style(|s| {
        s.set_background_color(Color::DARK_GRAY)
            .set_hover_color(Color::RED)
            .set_down_color(Color::BLUE)
            .set_self_halign(Alignment::Center)
            .set_size_pc(50.0, 20.0)
            .set_size_max_px(240.0, 90.0)
            .set_solid_fit()
            .set_aspect_ratio(16.0, 4.)
            .set_padding_px(10.0, 10.0, 10.0, 10.0);
    });
    let login_button_style = ui.create_button_style(|s| {
        s.set_margin_right_px(40.0);
    });
    let register_button_style = ui.create_button_style(|s| {
        s.set_margin_left_px(40.0);
    });

    // nodes
    ui.set_text_icon_asset_id(&icon_asset_id)
        .set_text_color(Color::WHITE)
        .root_mut()
        .add_style(window_style)
        .contents(|c| {
            // title container
            c.add_panel().add_style(container_style).contents(|c| {
                c.add_text("c y b e r l i t h");
            });

            // login button
            c.add_button("login_button")
                .set_as_default_button()
                .add_style(base_button_style)
                .add_style(login_button_style)
                .contents(|c| {
                    c.add_text("login");
                })
                .navigation(|n| {
                    n
                        .down_goes_to("register_button")
                        .right_goes_to("register_button");
                });

            // continue button
            c.add_button("register_button")
                .add_style(base_button_style)
                .add_style(register_button_style)
                .contents(|c| {
                    c.add_text("register");
                })
                .navigation(|n| {
                    n
                        .up_goes_to("login_button")
                        .left_goes_to("login_button");
                });
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui)
}