use bevy_ecs::world::World;

use render_egui::{
    egui,
    egui::{Frame, Modifiers, Resize, Ui},
    EguiUserTextures,
};

use crate::app::plugin::{LeftBottomTexture, LeftTopTexture, RightBottomTexture, RightTopTexture};

pub fn skeleton_builder(ui: &mut Ui, world: &mut World) {
    egui::SidePanel::left("left_work")
        .frame(Frame::side_top_panel(ui.style()).inner_margin(0.0))
        .resizable(true)
        .default_width(ui.available_width() * 0.5)
        .show_inside(ui, |ui| {
            egui::TopBottomPanel::top("left_top_work")
                .frame(Frame::side_top_panel(ui.style()).inner_margin(0.0))
                .resizable(true)
                .default_height(ui.available_height() * 0.5)
                .show_inside(ui, |ui| {
                    left_top_work(ui, world);
                });
            egui::CentralPanel::default() // left_bottom_work
                .frame(Frame::central_panel(ui.style()).inner_margin(0.0))
                .show_inside(ui, |ui| {
                    left_bottom_work(ui, world);
                });
        });

    egui::CentralPanel::default() // right work
        .frame(Frame::side_top_panel(ui.style()).inner_margin(0.0))
        .show_inside(ui, |ui| {
            egui::TopBottomPanel::top("right_top_work")
                .frame(Frame::side_top_panel(ui.style()).inner_margin(0.0))
                .resizable(true)
                .default_height(ui.available_height() * 0.5)
                .show_inside(ui, |ui| {
                    right_top_work(ui, world);
                });
            egui::CentralPanel::default() // right_bottom_work
                .frame(Frame::central_panel(ui.style()).inner_margin(0.0))
                .show_inside(ui, |ui| {
                    right_bottom_work(ui, world);
                });
        });
}

fn left_top_work(ui: &mut Ui, world: &mut World) {
    let left_top_texture = world.get_resource::<LeftTopTexture>().unwrap();
    let user_textures = world.get_resource::<EguiUserTextures>().unwrap();
    let Some(texture_id) = user_textures.texture_id(&left_top_texture.0) else {
        // The user texture may not be synced yet, return early.
        return;
    };
    ui.image(texture_id, ui.available_size());
}

fn left_bottom_work(ui: &mut Ui, world: &mut World) {
    let left_bottom_texture = world.get_resource::<LeftBottomTexture>().unwrap();
    let user_textures = world.get_resource::<EguiUserTextures>().unwrap();
    let Some(texture_id) = user_textures.texture_id(&left_bottom_texture.0) else {
        // The user texture may not be synced yet, return early.
        return;
    };
    ui.image(texture_id, ui.available_size());
}

fn right_top_work(ui: &mut Ui, world: &mut World) {
    let right_top_texture = world.get_resource::<RightTopTexture>().unwrap();
    let user_textures = world.get_resource::<EguiUserTextures>().unwrap();
    let Some(texture_id) = user_textures.texture_id(&right_top_texture.0) else {
        // The user texture may not be synced yet, return early.
        return;
    };
    ui.image(texture_id, ui.available_size());
}

fn right_bottom_work(ui: &mut Ui, world: &mut World) {
    let right_bottom_texture = world.get_resource::<RightBottomTexture>().unwrap();
    let user_textures = world.get_resource::<EguiUserTextures>().unwrap();
    let Some(texture_id) = user_textures.texture_id(&right_bottom_texture.0) else {
        // The user texture may not be synced yet, return early.
        return;
    };
    ui.image(texture_id, ui.available_size());
}
