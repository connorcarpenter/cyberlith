
use bevy_ecs::world::World;

use render_egui::{egui, egui::Frame};

use crate::app::ui::{
    workspaces::{skeleton_builder, text_editor},
    UiState, WorkspaceType,
    widgets::TabBarUiWidget,
};

pub fn center_panel(context: &egui::Context, world: &mut World) {
    egui::CentralPanel::default()
        .frame(Frame::none().inner_margin(0.0))
        .show(context, |ui| {
            egui::TopBottomPanel::top("tab_bar").show_inside(ui, |ui| {
                TabBarUiWidget::render_root(ui, world);
            });
            egui::CentralPanel::default() // workspace area
                .frame(Frame::central_panel(ui.style()).inner_margin(0.0))
                .show_inside(ui, |ui| {
                    let state = world.get_resource::<UiState>().unwrap();
                    match state.workspace_type {
                        WorkspaceType::None => {
                            ui.label("-");
                        }
                        WorkspaceType::SkeletonBuilder => {
                            skeleton_builder(ui, world);
                        }
                        WorkspaceType::TextEditor => {
                            text_editor(ui);
                        }
                    }
                });
        });
}
