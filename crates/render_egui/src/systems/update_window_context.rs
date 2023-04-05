use bevy_ecs::{
    change_detection::DetectChanges,
    system::{Res, ResMut},
};

use render_api::Window;

use crate::resources::{
    EguiContext, EguiInput, EguiOutput, EguiRenderOutput, EguiSettings, WindowSize,
};

pub fn update_window_context(
    mut egui_input: ResMut<EguiInput>,
    mut window_size: ResMut<WindowSize>,
    window: ResMut<Window>,
    egui_settings: Res<EguiSettings>,
) {
    if !window.is_changed() {
        return;
    }
    let new_window_size = WindowSize::new(
        window.physical_width() as f32,
        window.physical_height() as f32,
        window.scale_factor() as f32,
    );
    let width = new_window_size.physical_width
        / new_window_size.scale_factor
        / egui_settings.scale_factor as f32;
    let height = new_window_size.physical_height
        / new_window_size.scale_factor
        / egui_settings.scale_factor as f32;

    if width < 1.0 || height < 1.0 {
        return;
    }

    egui_input.inner_mut().screen_rect = Some(egui::Rect::from_min_max(
        egui::pos2(0.0, 0.0),
        egui::pos2(width, height),
    ));

    egui_input.inner_mut().pixels_per_point =
        Some(new_window_size.scale_factor * egui_settings.scale_factor as f32);

    *window_size = new_window_size;
}
