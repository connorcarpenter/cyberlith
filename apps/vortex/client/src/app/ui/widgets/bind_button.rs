use bevy_ecs::world::World;
use render_egui::egui;
use render_egui::egui::{Align, Direction, Frame, Layout, Ui};

pub fn render_bind_button(ui: &mut Ui, world: &mut World) {
    egui::CentralPanel::default()
        .show_inside(ui, |ui| {
            ui.with_layout(Layout::centered_and_justified(Direction::TopDown), |ui| {
                Frame::none()
                    .inner_margin(300.0)
                    .show(ui, |ui| {
                        ui.button("Bind to .skel File");
                    });
            });
        });
}