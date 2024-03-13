
use game_engine::{ui::{Alignment, Ui}, render::base::Color, asset::{AssetHandle, IconData}};

pub fn init_ui(text_handle: &AssetHandle<IconData>) -> Ui {
    let mut ui = Ui::new();

    let window_style = ui.create_panel_style(|s| {
        s
            //.set_background_color(Color::BLACK)
            .set_background_alpha(0.0)
            .set_padding_px(10.0, 10.0, 10.0, 10.0)
            .set_vertical()
            .set_row_between_px(10.0);
    });
    let container_style = ui.create_panel_style(|s| {
        s
            .set_background_alpha(0.0)
            .set_size_pc(100.0, 38.2)
            .set_solid_fit()
            .set_aspect_ratio(16.0, 4.5);
    });
    let base_button_style = ui.create_panel_style(|s| {
        s
            .set_background_color(Color::DARK_GRAY)
            .set_self_halign(Alignment::Center)
            .set_size_pc(50.0, 20.0)
            .set_size_max_px(240.0, 90.0)
            .set_solid_fit()
            .set_aspect_ratio(16.0, 4.5)
            .set_padding_px(10.0, 10.0, 10.0, 10.0);
    });
    let start_button_style = ui.create_panel_style(|s| {
        s.set_margin_right_px(40.0);
    });
    let continue_button_style = ui.create_panel_style(|s| {
        s.set_margin_left_px(40.0);
    });

    ui
        .set_text_icon_handle(&text_handle)
        .set_text_color(Color::WHITE)
        .root_mut()
        .add_style(window_style)
        .contents(|c| {
            // title container
            c
                .add_panel()
                .add_style(container_style)
                .contents(|c| {
                    c.add_text("c y b e r l i t h");
                });

            // start button
            c
                .add_panel()
                .add_style(base_button_style)
                .add_style(start_button_style)
                .contents(|c| {
                    c.add_text("start");
                });

            // continue button
            c
                .add_panel()
                .add_style(base_button_style)
                .add_style(continue_button_style)
                .contents(|c| {
                    c.add_text("continue");
                });
        });

    ui
}

pub fn write_ui(ui: Ui) -> Vec<u8> {
    let bytes = ui.write_json();

    // let byte_str = std::str::from_utf8(&bytes).unwrap();
    // info!("ui: {:?}", byte_str);

    bytes
}

pub fn read_ui(bytes: Vec<u8>) -> Ui {
    Ui::read_json(bytes)
}