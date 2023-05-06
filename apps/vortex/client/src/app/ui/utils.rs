use render_egui::egui;
use render_egui::egui::Ui;

pub fn ui_with_margin<R>(ui: &mut Ui, margin: f32, add_contents: impl FnOnce(&mut Ui) -> R) {
    egui::Frame::none()
        .inner_margin(margin)
        .show(ui, |ui| add_contents(ui));
}