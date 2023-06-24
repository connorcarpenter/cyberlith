use bevy_ecs::world::World;

use render_egui::{
    egui,
    egui::{Frame, Ui},
    EguiUserTextures,
};

use crate::app::plugin::WorkspaceTexture;

pub fn skeleton_builder(ui: &mut Ui, world: &mut World) {
    egui::CentralPanel::default() // left_bottom_work
        .frame(Frame::central_panel(ui.style()).inner_margin(0.0))
        .show_inside(ui, |ui| {
            work_panel(ui, world);
        });
}

fn work_panel(ui: &mut Ui, world: &mut World) {
    let workspace_texture = world.get_resource::<WorkspaceTexture>().unwrap();
    let user_textures = world.get_resource::<EguiUserTextures>().unwrap();
    let Some(texture_id) = user_textures.texture_id(&workspace_texture.0) else {
        // The user texture may not be synced yet, return early.
        return;
    };
    ui.image(texture_id, ui.available_size());
}