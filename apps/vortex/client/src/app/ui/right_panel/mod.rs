use bevy_ecs::world::World;

use render_egui::{
    egui,
    egui::{Frame, Margin, Ui},
};

use vortex_proto::components::FileExtension;

use crate::app::resources::toolbar::Toolbar;

pub fn render_tool_bar(ui: &mut Ui, world: &mut World, file_ext: FileExtension) {
    egui::SidePanel::right("right_panel")
        .frame(Frame::side_top_panel(ui.style()).inner_margin(Margin {
            left: 3.0,
            right: 1.0,
            top: 2.0,
            bottom: 2.0,
        }))
        .resizable(false)
        .default_width(26.0)
        .show_inside(ui, |ui| {
            ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
            ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                Toolbar::render(ui, world, file_ext);

                ui.allocate_space(ui.available_size());
            });
            ui.allocate_space(ui.available_size());
        });
}
