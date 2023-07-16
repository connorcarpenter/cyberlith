use bevy_ecs::{
    system::{Query, Res, ResMut, SystemState},
    world::World,
};

use input::Input;
use math::Vec2;
use render_api::{
    Assets,
    base::CpuTexture2D,
    components::{Camera, Projection, Transform},
};
use render_egui::{
    egui,
    egui::{Frame, Id, Image, pos2, Rect, Ui},
    EguiUserTextures,
};

use crate::app::{
    resources::canvas_manager::CanvasManager,
    ui::UiState,
};

pub fn show_canvas(ui: &mut Ui, world: &mut World) {
    egui::CentralPanel::default()
        .frame(Frame::central_panel(ui.style()).inner_margin(0.0))
        .show_inside(ui, |ui| {
            work_panel(ui, world);
        });
}

fn work_panel(ui: &mut Ui, world: &mut World) {
    let did_resize = resize_finished(ui, world, "left_panel");

    let mut system_state: SystemState<(
        Res<CanvasManager>,
        ResMut<Assets<CpuTexture2D>>,
        ResMut<EguiUserTextures>,
        ResMut<UiState>,
        ResMut<Input>,
        Query<(&mut Camera, &mut Transform, &mut Projection)>,
    )> = SystemState::new(world);
    let (
        canvas_state,
        mut textures,
        mut user_textures,
        mut ui_state,
        mut input,
        mut camera_query,
    ) = system_state.get_mut(world);

    // change textures
    let texture_handle = canvas_state.canvas_texture();
    let Some(texture_id) = user_textures.texture_id(&texture_handle) else {
        // The user texture may not be synced yet, return early.
        return;
    };
    let top_left = ui.min_rect().min;
    if ui_state.canvas_coords.is_none() {
        ui_state.canvas_coords = Some(top_left);
        input.set_mouse_offset(top_left.x, top_left.y);
    }
    let texture_size = ui.available_size();

    if did_resize {
        ui_state.canvas_coords = Some(top_left);
        input.set_mouse_offset(top_left.x + 1.0, top_left.y + 1.0);

        // This is the texture that will be rendered to.
        let texture_width = texture_size.x as u32;
        let texture_height = texture_size.y as u32;
        let new_texture = CpuTexture2D::from_size(texture_width, texture_height);

        textures.set(&texture_handle, new_texture);
        user_textures.mark_texture_changed(&texture_handle);

        // Update the camera to match the new texture size.
        let native_texture_size = Vec2::new(texture_size.x, texture_size.y);
        canvas_state.update_camera_viewports(native_texture_size, &mut camera_query);
    }

    if canvas_state.is_visible() {
        let image =
            Image::new(texture_id, texture_size).uv(Rect::from_min_max(pos2(0.0, 1.0), pos2(1.0, 0.0)));
        ui.add_enabled(false, image);
    }

    system_state.apply(world);
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

    return false;
}
