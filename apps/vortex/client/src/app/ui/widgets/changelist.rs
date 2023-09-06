use bevy_ecs::{
    system::{Res, SystemState},
    world::World,
};

use render_egui::egui::{Align, Layout, Ui};

use crate::app::{resources::file_manager::FileManager, ui::widgets::ChangelistRowUiWidget};

pub struct ChangelistUiWidget;

impl ChangelistUiWidget {
    pub fn render_root(ui: &mut Ui, world: &mut World) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            let mut system_state: SystemState<Res<FileManager>> = SystemState::new(world);
            let file_manager = system_state.get(world);

            let entities = file_manager.changelist_entities();
            for entity in entities {
                ChangelistRowUiWidget::render_row(ui, world, entity);
            }
        });
    }
}
