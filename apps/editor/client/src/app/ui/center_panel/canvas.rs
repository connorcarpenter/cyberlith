use bevy_ecs::{
    system::{Query, Res, ResMut, SystemState},
    world::World,
};

use input::Input;
use math::Vec2;
use render_api::{
    base::CpuTexture2D,
    components::{Camera, Projection, Transform},
};
use render_egui::{
    egui,
    egui::{pos2, Color32, Frame, Id, Image, Rect, Ui},
    EguiUserTextures,
};
use storage::Storage;

use crate::app::{
    resources::{
        camera_manager::CameraManager, canvas::Canvas, icon_manager::IconManager,
        tab_manager::TabManager,
    },
    ui::UiState,
};

pub fn render_canvas(ui: &mut Ui, world: &mut World) {
    egui::CentralPanel::default() // canvas area
        .frame(Frame::central_panel(ui.style()).inner_margin(0.0))
        .show_inside(ui, |ui| {
            if is_resizing(ui, world, "left_panel") {
                let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
                ui_state.resized_window = true;
            }

            let mut system_state: SystemState<(
                ResMut<TabManager>,
                ResMut<Canvas>,
                ResMut<CameraManager>,
                Res<IconManager>,
                ResMut<Storage<CpuTexture2D>>,
                ResMut<EguiUserTextures>,
                ResMut<UiState>,
                ResMut<Input>,
                Query<(&mut Camera, &mut Transform, &mut Projection)>,
            )> = SystemState::new(world);
            let (
                mut tab_manager,
                mut canvas,
                mut camera_manager,
                icon_manager,
                mut textures,
                mut user_textures,
                mut ui_state,
                mut input,
                mut camera_query,
            ) = system_state.get_mut(world);

            if ui_state.canvas_coords.is_none() {
                ui_state.resized_window = true;
            }

            let mut frame = Frame::central_panel(ui.style()).inner_margin(1.0);
            if canvas.is_visible() {
                if tab_manager.has_focus() {
                    frame = frame.fill(Color32::DARK_GRAY);
                } else {
                    frame = frame.fill(Color32::BLACK);
                }
            }

            egui::CentralPanel::default()
                .frame(frame)
                .show_inside(ui, |ui| {
                    // change textures
                    let texture_handle = canvas.texture_handle();
                    let Some(texture_id) = user_textures.texture_id(&texture_handle) else {
                        // The user texture may not be synced yet, return early.
                        return;
                    };
                    let top_left = ui.min_rect().min;
                    let mut texture_size = ui.available_size();
                    texture_size.y -= 3.0;

                    if !input.has_canvas_properties() {
                        input.update_canvas_properties(top_left.x + 1.0, top_left.y + 1.0);
                    }
                    if ui_state.resized_window {
                        let last_texture_size = canvas.texture_size();
                        let current_texture_size = Vec2::new(texture_size.x, texture_size.y);

                        if last_texture_size != current_texture_size {
                            ui_state.canvas_coords = Some(top_left);
                            input.update_canvas_properties(top_left.x + 1.0, top_left.y + 1.0);

                            // This is the texture that will be rendered to.
                            let texture_width = texture_size.x as u32;
                            let texture_height = texture_size.y as u32;
                            let new_texture =
                                CpuTexture2D::from_size(texture_width, texture_height);

                            textures.set(&texture_handle, new_texture);
                            user_textures.mark_texture_changed(&texture_handle);

                            // Update the camera to match the new texture size.
                            canvas.update_texture_size(current_texture_size);
                            camera_manager
                                .update_camera_viewports(current_texture_size, &mut camera_query);
                            icon_manager
                                .update_camera_viewport(current_texture_size, &mut camera_query);

                            // queue recalc of shapes
                            camera_manager.recalculate_3d_view();
                            canvas.queue_resync_shapes_light();

                            // info!(
                            //     "resized window to: (width: {:?}, height: {:?})",
                            //     texture_width, texture_height
                            // );
                        }

                        ui_state.resized_window = false;
                    }

                    if canvas.is_visible() {
                        let image = Image::new(texture_id, texture_size)
                            .bg_fill(Color32::BLACK)
                            .uv(Rect::from_min_max(pos2(0.0, 1.0), pos2(1.0, 0.0)))
                            .sense(egui::Sense::click_and_drag());
                        let canvas_response = ui.add_enabled(true, image);

                        if canvas_response.clicked() || canvas_response.dragged() {
                            canvas_response.request_focus();
                            tab_manager.set_focus(true);
                        } else if canvas_response.clicked_elsewhere() {
                            canvas_response.surrender_focus();
                            tab_manager.set_focus(false);
                        }

                        let has_focus = tab_manager.has_focus();
                        input.set_enabled(has_focus);
                    } else {
                        input.set_enabled(false);
                    }
                });

            system_state.apply(world);
        });
}

fn is_resizing(ui: &Ui, world: &mut World, id_impl: impl Into<Id>) -> bool {
    let Some(mut ui_state) = world.get_resource_mut::<UiState>() else {
        return false;
    };

    let id: Id = id_impl.into();
    let resize_id: Id = id.with("__resize");
    let is_resizing = ui.memory(|mem| mem.is_being_dragged(resize_id));

    if ui_state.dragging_side_panel && !is_resizing {
        ui_state.dragging_side_panel = false;
    }
    if !ui_state.dragging_side_panel && is_resizing {
        ui_state.dragging_side_panel = true;
    }

    return is_resizing;
}
