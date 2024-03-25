use game_engine::{ui::{Alignment, Ui}, render::base::Color, asset::{AssetId, ETag}};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, Ui) {
    // config
    let ui_name = "temp";
    let ui_asset_id_str = "57rwgx";//AssetId::get_random().as_string(); // keep this around to generate new AssetIds if needed!
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
            .set_background_alpha(1.)
            .set_background_color(Color::BLACK)
        ;
    });
    let main_container_style = ui.create_panel_style(|s| {
        s
            .set_background_alpha(1.)
            .set_background_color(Color::BLACK)
            .set_size_pc(100., 100.)
            .set_solid_fit()
            .set_aspect_ratio(16., 9.);
    });
    let title_container_style = ui.create_panel_style(|s| {
        s
            .set_background_alpha(1.)
            .set_background_color(Color::RED)
            .set_size_pc(100., 33.)
            .set_vertical()
            .set_children_valign(Alignment::Center)
        ;
    });
    let body_container_style = ui.create_panel_style(|s| {
        s
            .set_background_alpha(1.)
            .set_background_color(Color::BLUE)
            .set_size_pc(100., 67.)
        ;
    });
    let base_textbox_style = ui.create_textbox_style(|s| {
        s
            .set_background_color(Color::GRAY)
            .set_width_pc(45.)
            .set_height_vp(7.0)
        ;
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

                });

                // body container
                c.add_panel().add_style(body_container_style).contents(|c| {
                    c.add_textbox("my_textbox").add_style(base_textbox_style);
                });
            });

        });

    (ui_name.to_string(), ui_asset_id, ui_etag, ui)
}