use bevy_ecs::{system::{Query, Res, ResMut, SystemState}, world::World};
use bevy_log::info;

use input::Input;
use math::Vec3;
use render_api::{Assets, base::CpuTexture2D, components::{Camera, Viewport}};
use render_api::components::{OrthographicProjection, Projection, Transform};
use render_egui::{
    egui,
    egui::{Frame, Id, Image, pos2, Rect, Ui, Vec2},
    EguiUserTextures,
};

use crate::app::{plugin::WorkspaceTexture, resources::global::Global, ui::UiState};

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
        Res<Global>,
        ResMut<Assets<CpuTexture2D>>,
        ResMut<EguiUserTextures>,
        Res<WorkspaceTexture>,
        ResMut<UiState>,
        ResMut<Input>,
        Query<(&mut Camera, &mut Transform, &mut Projection)>,
    )> = SystemState::new(world);
    let (global, mut textures, mut user_textures, workspace_texture, mut ui_state, mut input, mut camera_query) = system_state.get_mut(world);

    let texture_handle = workspace_texture.0;
    let Some(texture_id) = user_textures.texture_id(&texture_handle) else {
        // The user texture may not be synced yet, return early.
        return;
    };
    let top_left = ui.min_rect().min;
    if ui_state.workspace_coords.is_none() {
        ui_state.workspace_coords = Some(top_left);
        input.set_mouse_offset(top_left.x, top_left.y);
    }
    let texture_size = ui.available_size();
    let image = Image::new(texture_id, texture_size)
        .uv(Rect::from_min_max(pos2(0.0, 1.0), pos2(1.0, 0.0)));
    ui.add(image);

    if did_resize {
        info!("Resize panel finished! New size: {:?}", texture_size);

        ui_state.workspace_coords = Some(top_left);
        input.set_mouse_offset(top_left.x + 1.0, top_left.y + 1.0);

        // This is the texture that will be rendered to.
        let texture_width = texture_size.x as u32;
        let texture_height = texture_size.y as u32;
        let new_texture = CpuTexture2D::from_size(texture_width, texture_height);

        textures.set(&texture_handle, new_texture);
        user_textures.mark_texture_changed(&texture_handle);

        // Update the camera to match the new texture size.
        let Some(camera_entity) = global.workspace_camera else {
            return;
        };
        let Ok((mut camera, mut transform, mut projection)) = camera_query.get_mut(camera_entity) else {
            return;
        };
        camera.viewport = Some(Viewport::new_at_origin(texture_width, texture_height));
        *transform = Transform::from_xyz(
            texture_width as f32 * 0.5,
            texture_height as f32 * 0.5,
            -1.0,
        )
            .looking_at(
                Vec3::new(
                    texture_width as f32 * 0.5,
                    texture_height as f32 * 0.5,
                    0.0,
                ),
                Vec3::NEG_Y,
            );
        *projection = Projection::Orthographic(OrthographicProjection {
            height: texture_height as f32,
            near: 0.0,
            far: 10.0,
        });
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