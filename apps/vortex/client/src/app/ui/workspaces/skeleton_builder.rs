use bevy_ecs::{system::{Res, ResMut, SystemState}, world::World};
use bevy_log::info;

use render_api::{Assets, base::CpuTexture2D};
use render_egui::{
    egui,
    egui::{Frame, Id, Ui},
    EguiUserTextures,
};

use crate::app::{plugin::WorkspaceTexture, ui::UiState};

pub fn skeleton_builder(ui: &mut Ui, world: &mut World) {
    egui::CentralPanel::default()
        .frame(Frame::central_panel(ui.style()).inner_margin(0.0))
        .show_inside(ui, |ui| {
            work_panel(ui, world);
        });
}

fn work_panel(ui: &mut Ui, world: &mut World) {
    let did_resize = resize_finished(ui, world, "left_panel");

    let mut system_state: SystemState<(
        ResMut<Assets<CpuTexture2D>>,
        ResMut<EguiUserTextures>,
        Res<WorkspaceTexture>,
    )> = SystemState::new(world);
    let (mut textures, mut user_textures, workspace_texture) = system_state.get_mut(world);

    let texture_handle = workspace_texture.0;
    let Some(texture_id) = user_textures.texture_id(&texture_handle) else {
        // The user texture may not be synced yet, return early.
        return;
    };
    let texture_size = ui.available_size();
    ui.image(texture_id, texture_size);

    if did_resize {
        info!("Resize panel finished!");

        // This is the texture that will be rendered to.
        let new_texture = CpuTexture2D::from_size(texture_size.x as u32, texture_size.y as u32);

        textures.set(&texture_handle, new_texture);
        user_textures.mark_texture_changed(&texture_handle);
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