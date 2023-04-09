
mod tab_bar;
use tab_bar::tab_bar;

use bevy_ecs::{change_detection::ResMut, world::World};

use render_egui::egui;

use crate::app::ui::{UiState, WorkspaceType, workspaces::{skeleton_builder, text_editor}};

pub fn center_panel(
    context: &egui::Context,
    world: &mut World,
) {
    egui::CentralPanel::default()
        .show(context, |ui| {
            tab_bar(ui, world);
            let state = world.get_resource::<UiState>().unwrap();
            match state.workspace_type {
                WorkspaceType::None => {
                    // nothing
                }
                WorkspaceType::SkeletonBuilder => {
                    skeleton_builder(ui);
                }
                WorkspaceType::TextEditor => {
                    text_editor(ui);
                }
            }
        });
}