use bevy_ecs::world::World;

use render_egui::{egui, egui::{Ui, Modifiers, Resize}, EguiUserTextures};

use crate::app::plugin::LeftTopTexture;

pub fn skeleton_builder(
    ui: &mut Ui,
    world: &mut World,
) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        egui::SidePanel::left("left_work")
            .resizable(true)
            .default_width(150.0)
            .show_inside(ui, |ui| {
                egui::TopBottomPanel::top("left_top_work")
                    .resizable(true)
                    .show_inside(ui, |ui| {
                        left_top_work(ui, world);
                    });
                egui::CentralPanel::default()
                    .show_inside(ui, |ui| {
                        left_bottom_work(ui);
                    });
            });
        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                egui::TopBottomPanel::top("right_top_work")
                    .resizable(true)
                    .show_inside(ui, |ui| {
                        right_top_work(ui);
                    });
                egui::CentralPanel::default()
                    .show_inside(ui, |ui| {
                        right_bottom_work(ui);
                    });
            });
    });
}

fn left_top_work(
    ui: &mut Ui,
    world: &mut World,
) {
    let left_top_texture = world.get_resource::<LeftTopTexture>().unwrap();
    let user_textures = world.get_resource::<EguiUserTextures>().unwrap();
    let Some(texture_id) = user_textures.texture_id(&left_top_texture.0) else {
        // The user texture may not be synced yet, return early.
        return;
    };
    ui.image(texture_id, [300.0, 300.0]);
}

fn left_bottom_work(
    ui: &mut Ui,
) {
    multiline(ui, "left_bottom_work");
}

fn right_top_work(
    ui: &mut Ui,
) {
    multiline(ui, "right_top_work");
}

fn right_bottom_work(
    ui: &mut Ui,
) {
    multiline(ui, "right_bottom_work");
}

fn multiline(ui: &mut Ui, text: &str) {
    Resize::default().show(ui, |ui| {
        lorem_ipsum(ui, text);
    });
}

fn lorem_ipsum(ui: &mut Ui, text: &str) {
    ui.with_layout(
        egui::Layout::top_down(egui::Align::LEFT)
            .with_cross_justify(true),
        |ui| {
            ui.label(egui::RichText::new(text).weak());
        },
    );
}