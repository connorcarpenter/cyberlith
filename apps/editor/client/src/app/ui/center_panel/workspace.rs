
use render_egui::{egui, egui::{Ui, Modifiers, Resize}};

pub fn workspace(
    ui: &mut Ui,
) {
    egui::CentralPanel::default().show_inside(ui, |ui| {
        left_work(ui);
        right_work(ui);
    });
}

fn left_work(
    ui: &mut Ui,
) {
    egui::SidePanel::left("left_work")
        .resizable(true)
        .default_width(150.0)
        .show_inside(ui, |ui| {
            left_top_work(ui);
            left_bottom_work(ui);
        });
}

fn right_work(
    ui: &mut Ui,
) {
    egui::CentralPanel::default()
        .show_inside(ui, |ui| {
            right_top_work(ui);
            right_bottom_work(ui);
        });
}

fn left_top_work(
    ui: &mut Ui,
) {
    egui::TopBottomPanel::top("left_top_work")
        .resizable(true)
        .show_inside(ui, |ui| {
            multiline(ui, "left_top_work");
        });
}

fn left_bottom_work(
    ui: &mut Ui,
) {
    egui::CentralPanel::default()
        .show_inside(ui, |ui| {
            multiline(ui, "left_bottom_work");
        });
}

fn right_top_work(
    ui: &mut Ui,
) {
    egui::TopBottomPanel::top("right_top_work")
        .resizable(true)
        .show_inside(ui, |ui| {
            multiline(ui, "right_top_work");
        });
}

fn right_bottom_work(
    ui: &mut Ui,
) {
    egui::CentralPanel::default()
        .show_inside(ui, |ui| {
            multiline(ui, "right_bottom_work");
        });
}

fn multiline(ui: &mut Ui, text: &str) {
    Resize::default().show(ui, |ui| {
        lorem_ipsum(ui, text);
    });
}

fn lorem_ipsum(ui: &mut Ui, text: &str) {
    ui.with_layout(
        egui::Layout::top_down(egui::Align::LEFT)
            .with_cross_justify(true),
        |ui| {
            ui.label(egui::RichText::new(text).weak());
        },
    );
}