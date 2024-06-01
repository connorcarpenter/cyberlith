use game_engine::{
    asset::{AssetId, ETag},
    render::base::Color,
};
use ui_builder::{Alignment, UiConfig, UiConfigBuild};
use ui_runner_config::{EmailValidation, PasswordValidation, UsernameValidation};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "register";
    let ui_asset_id_str = "rckneg"; //AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
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
            .set_children_halign(Alignment::Center);
    });
    let main_container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_as_viewport()
            .set_size_pc(100., 100.)
            .set_solid_fit()
            .set_aspect_ratio(16., 9.)
            .set_self_halign(Alignment::Center);
    });
    let title_container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_size_pc(100., 16.)
            .set_vertical()
            .set_children_valign(Alignment::Start);
    });
    let title_text_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            .set_size_pc(90.0)
            .set_text_color(Color::WHITE);
    });
    let body_container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_size_pc(100., 84.)
            .set_vertical()
            .set_children_valign(Alignment::Start)
            .set_row_between_vp(0.5);
    });
    let heading_container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_width_pc(100.0)
            .set_height_pc(10.0)
            .set_horizontal();
    });
    let heading_container_left_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_width_pc(50.0)
            .set_height_pc(100.0);
    });
    let heading_container_right_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_width_pc(50.0)
            .set_height_pc(70.0);
    });
    let heading_text_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            .set_size_vp(7.0)
            .set_margin_left_vp(2.0)
            .set_self_halign(Alignment::Start)
            .set_text_color(Color::WHITE);
    });
    let button_container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_margin_left_vp(4.)
            .set_margin_top_vp(2.)
            .set_self_halign(Alignment::Start)
            .set_horizontal()
            .set_children_halign(Alignment::Start);
    });
    let spinner_style = ui_config.create_spinner_style(|s| {
        s.set_background_alpha(0.)
            .set_size_vp(4.0, 4.0)
            .set_margin_left_vp(2.0)
            .set_self_halign(Alignment::End)
            .set_self_valign(Alignment::Center)
            .set_spinner_color(Color::WHITE);
    });
    let error_output_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            .set_size_vp(2.0)
            .set_margin_left_vp(2.0)
            .set_self_halign(Alignment::End)
            .set_self_valign(Alignment::Center)
            .set_text_color(Color::RED);
    });
    let base_button_text_style = ui_config.create_text_style(|s| {
        s.set_size_vp(5.0)
            .set_self_halign(Alignment::Center)
            .set_self_valign(Alignment::Center)
            .set_margin_vp(1.0, 1.0, 1.0, 1.0)
            .set_text_color(Color::WHITE);
    });
    let base_button_style = ui_config.create_button_style(|s| {
        s.set_background_color(Color::DARK_GRAY)
            .set_hover_color(Color::RED)
            .set_down_color(Color::BLUE);
    });
    let submit_button_style = ui_config.create_button_style(|s| {
        s.set_parent_style(base_button_style)
            .set_self_halign(Alignment::Start);
    });
    let register_button_style = ui_config.create_button_style(|s| {
        s.set_parent_style(base_button_style)
            .set_self_halign(Alignment::End)
            .set_margin_right_vp(1.0);
    });
    let base_label_style = ui_config.create_text_style(|s| {
        s.set_background_alpha(0.)
            .set_size_vp(5.)
            .set_self_halign(Alignment::Start)
            .set_margin_left_vp(4.0)
            .set_text_color(Color::WHITE);
    });
    let base_textbox_style = ui_config.create_textbox_style(|s| {
        s.set_background_color(Color::GRAY)
            .set_hover_color(Color::RED)
            .set_active_color(Color::BLUE)
            .set_selection_color(Color::DARK_BLUE)
            .set_size_pc(45., 7.)
            .set_self_halign(Alignment::Start)
            .set_margin_left_vp(4.0);
    });

    // nodes
    ui_config
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
                c.add_panel().set_style(body_container_style).contents(|c| {
                    // heading container
                    c.add_panel()
                        .set_style(heading_container_style)
                        .contents(|c| {
                            // heading container left
                            c.add_panel()
                                .set_style(heading_container_left_style)
                                .contents(|c| {
                                    c.add_text("register your account")
                                        .set_style(heading_text_style);
                                });

                            // heading container right
                            c.add_panel()
                                .set_style(heading_container_right_style)
                                .contents(|c| {
                                    // login button
                                    c.add_button("login_button")
                                        .set_style(register_button_style)
                                        .contents(|c| {
                                            c.add_text("login").set_style(base_button_text_style);
                                        })
                                        .navigation(|n| {
                                            n.left_goes_to("username_textbox")
                                                .down_goes_to("username_textbox")
                                                .tab_goes_to("username_textbox");
                                        });
                                });
                        });

                    // username input
                    // text
                    c.add_text("username:").set_style(base_label_style);
                    // text-edit
                    c.add_textbox("username_textbox")
                        .validation::<UsernameValidation>()
                        .set_style(base_textbox_style)
                        .set_as_first_input()
                        .navigation(|n| {
                            n.up_goes_to("login_button")
                                .down_goes_to("email_textbox")
                                .tab_goes_to("email_textbox")
                                .right_goes_to("login_button");
                        });

                    // email input
                    // text
                    c.add_text("email address:").set_style(base_label_style);
                    // text-edit
                    c.add_textbox("email_textbox")
                        .validation::<EmailValidation>()
                        .set_style(base_textbox_style)
                        .navigation(|n| {
                            n.up_goes_to("username_textbox")
                                .down_goes_to("password_textbox")
                                .tab_goes_to("password_textbox")
                                .right_goes_to("login_button");
                        });

                    // password input
                    // text
                    c.add_text("password:").set_style(base_label_style);
                    // text-edit
                    c.add_textbox("password_textbox")
                        .set_as_password()
                        .validation::<PasswordValidation>()
                        .set_style(base_textbox_style)
                        .navigation(|n| {
                            n.up_goes_to("email_textbox")
                                .down_goes_to("confirm_password_textbox")
                                .tab_goes_to("confirm_password_textbox")
                                .right_goes_to("login_button");
                        });

                    // confirm password input
                    // text
                    c.add_text("confirm password:").set_style(base_label_style);
                    // text-edit
                    c.add_textbox("confirm_password_textbox")
                        .set_as_password()
                        .validation::<PasswordValidation>()
                        .set_style(base_textbox_style)
                        .navigation(|n| {
                            n.up_goes_to("password_textbox")
                                .down_goes_to("submit_button")
                                .tab_goes_to("submit_button")
                                .right_goes_to("login_button");
                        });

                    c.add_panel()
                        .set_style(button_container_style)
                        .contents(|c| {
                            // submit button
                            c.add_button("submit_button")
                                .set_style(submit_button_style)
                                .contents(|c| {
                                    c.add_text("submit").set_style(base_button_text_style);
                                })
                                .navigation(|n| {
                                    n.up_goes_to("confirm_password_textbox")
                                        .right_goes_to("login_button")
                                        .tab_goes_to("login_button");
                                });

                            // spinner
                            c.add_spinner("spinner")
                                .set_style(spinner_style)
                                .set_visible(false);

                            // error output
                            c.add_text_with_id("", "error_output_text")
                                .set_style(error_output_style);
                        });
                });
            });
        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
