use bevy_ecs::world::World;
use bevy_log::info;

use render_egui::{
    egui,
    egui::{Frame, Ui},
    EguiUserTextures,
};
use render_egui::egui::Id;

use crate::app::plugin::WorkspaceTexture;
use crate::app::ui::UiState;

pub fn skeleton_builder(ui: &mut Ui, world: &mut World) {
    egui::CentralPanel::default()
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

    if resize_finished(ui, world, "left_panel") {
        info!("Resize panel finished!");
    }
}

fn resize_finished(ui: &Ui, world: &mut World, id_impl: impl Into<Id>) -> bool {
    let Some(mut ui_state) = world.get_resource_mut::<UiState>() else {
        return false;
    };

    let id: Id = id_impl.into();
    let resize_id: Id = id.with("__resize");
    let is_resizing = ui.memory(|mem| mem.is_being_dragged(resize_id));

    if ui_state.dragging_side_panel && !is_resizing {
        ui_state.dragging_side_panel = false;
        return true;
    }
    if !ui_state.dragging_side_panel && is_resizing {
        ui_state.dragging_side_panel = true;
    }

    return false
}