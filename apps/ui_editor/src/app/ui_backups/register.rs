use game_engine::{ui::{Alignment, Ui}, render::base::Color, asset::{AssetId, ETag}};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, Ui) {
    // config
    let ui_name = "register";
    let ui_asset_id_str = "rckneg";//AssetId::get_random().as_string(); // keep this around to generate new AssetIds if needed!
    let icon_asset_id_str = "34mvvk"; // this probably shouldn't change, it's the text font
    let ui_etag = ETag::new_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(&ui_asset_id_str).unwrap();
    let icon_asset_id = AssetId::from_str(icon_asset_id_str).unwrap();

    // Create UI !
    let mut ui = Ui::new();

    // styles
    let window_style = ui.create_panel_style(|s| {
        s
            .set_background_alpha(0.);
    });
    let main_container_style = ui.create_panel_style(|s| {
        s
            .set_background_alpha(0.)
            .set_size_pc(100., 100.)
            .set_solid_fit()
            .set_aspect_ratio(16., 9.);
    });
    let title_container_style = ui.create_panel_style(|s| {
        s
            .set_background_alpha(0.)
            .set_size_pc(100., 16.)
            .set_vertical()
            .set_children_valign(Alignment::Start)
        ;
    });
    let title_text_style = ui.create_text_style(|s| {
        s
            .set_background_alpha(0.)
            .set_size_pc(90.0);
    });
    let body_container_style = ui.create_panel_style(|s| {
        s
            .set_background_alpha(0.)
            .set_size_pc(100., 84.)
            .set_vertical()
            .set_children_valign(Alignment::Start)
            .set_row_between_px(5.0);
    });
    let heading_container_style = ui.create_panel_style(|s| {
        s
            .set_background_alpha(0.)
            .set_width_pc(100.0)
            .set_height_pc(10.0)
            .set_horizontal();
    });
    let heading_container_left_style = ui.create_panel_style(|s| {
        s
            .set_background_alpha(0.)
            .set_width_pc(50.0)
            .set_height_pc(100.0);
    });
    let heading_container_right_style = ui.create_panel_style(|s| {
        s
            .set_background_alpha(0.)
            .set_width_pc(50.0)
            .set_height_pc(70.0);
    });
    let heading_text_style = ui.create_text_style(|s| {
        s
            .set_background_alpha(0.)
            .set_size_vp(7.0)
            .set_margin_left_px(20.0)
            .set_self_halign(Alignment::Start);
    });
    let base_button_text_style = ui.create_text_style(|s| {
        s
            .set_size_vp(5.0)
            .set_self_halign(Alignment::Center)
            .set_self_valign(Alignment::Center)
            .set_margin_px(10.0, 10.0, 10.0, 10.0);
    });
    let base_button_style = ui.create_button_style(|s| {
        s.set_background_color(Color::DARK_GRAY)
            .set_hover_color(Color::RED)
            .set_down_color(Color::BLUE);
    });
    let submit_button_style = ui.create_button_style(|s| {
        s
            .set_self_halign(Alignment::Start)
            .set_margin_left_px(40.);
    });
    let register_button_style = ui.create_button_style(|s| {
        s
            .set_self_halign(Alignment::End)
            .set_margin_right_px(10.0);
    });
    let base_label_style = ui.create_text_style(|s| {
        s
            .set_background_alpha(0.)
            .set_size_vp(5.)
            .set_self_halign(Alignment::Start)
            .set_margin_left_px(40.0);
    });
    let base_textbox_style = ui.create_textbox_style(|s| {
        s
            .set_background_color(Color::GRAY)
            .set_size_pc(45., 7.)
            .set_self_halign(Alignment::Start)
            .set_margin_left_px(40.0);
    });

    // nodes
    ui.set_text_icon_asset_id(&icon_asset_id)
        .set_text_color(Color::WHITE)
        .root_mut()
        .add_style(window_style)
        .contents(|c| {

            // main container
            c.add_panel().add_style(main_container_style).contents(|c| {
                // title container
                c.add_panel().add_style(title_container_style).contents(|c| {
                    c.add_text("c y b e r l i t h").add_style(title_text_style);
                });

                // body container
                c.add_panel().add_style(body_container_style).contents(|c| {
                    // heading container
                    c.add_panel().add_style(heading_container_style).contents(|c| {
                        // heading container left
                        c.add_panel().add_style(heading_container_left_style).contents(|c| {
                            c.add_text("register your account").add_style(heading_text_style);
                        });

                        // heading container right
                        c.add_panel().add_style(heading_container_right_style).contents(|c| {
                            // register button
                            c.add_button("login_button")
                                .add_style(base_button_style)
                                .add_style(register_button_style)
                                .contents(|c| {
                                    c.add_text("login").add_style(base_button_text_style);
                                });
                        });

                    });

                    // username input
                    // text
                    c.add_text("username:").add_style(base_label_style);
                    // text-edit
                    c.add_textbox("username_textbox").add_style(base_textbox_style);

                    // email input
                    // text
                    c.add_text("email address:").add_style(base_label_style);
                    // text-edit
                    c.add_textbox("email_textbox").add_style(base_textbox_style);

                    // password input
                    // text
                    c.add_text("password:").add_style(base_label_style);
                    // text-edit
                    c.add_textbox("password_textbox").add_style(base_textbox_style);

                    // confirm password input
                    // text
                    c.add_text("confirm password:").add_style(base_label_style);
                    // text-edit
                    c.add_textbox("confirm_password_textbox").add_style(base_textbox_style);

                    // submit button
                    c.add_button("submit_button")
                        .add_style(base_button_style)
                        .add_style(submit_button_style)
                        .contents(|c| {
                            c.add_text("submit").add_style(base_button_text_style);
                        });

                });
            });

        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui)
}