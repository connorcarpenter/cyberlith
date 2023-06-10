use bevy_ecs::{world::World, system::{Res, SystemState}};

use render_egui::egui::{Align, Layout, Ui};

use crate::app::{
    resources::global::Global,
    ui::widgets::ChangelistRowUiWidget,
};

pub struct ChangelistUiWidget;

impl ChangelistUiWidget {
    pub fn render_root(ui: &mut Ui, world: &mut World) {
        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {

            let mut system_state: SystemState<Res<Global>> = SystemState::new(world);
            let global = system_state.get(world);

            let mut entities = Vec::new();
            for (_, entity) in global.changelist.iter() {
                entities.push(*entity);
            }
            for entity in entities {
                ChangelistRowUiWidget::render_row(ui, world, entity);
            }
        });
    }
}
