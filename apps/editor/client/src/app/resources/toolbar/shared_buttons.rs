use bevy_ecs::{
    system::{Res, ResMut, SystemState},
    world::World,
};

use render_egui::egui::Ui;

use crate::app::resources::{
    canvas::Canvas, edge_manager::EdgeManager, file_manager::FileManager, tab_manager::TabManager,
    toolbar::Toolbar,
};

pub(crate) fn button_toggle_edge_angle_visibility(ui: &mut Ui, world: &mut World) {
    // toggle edge angle visibility
    let response = Toolbar::button(ui, "📐", "Toggle edge angle visibility", true);
    if response.clicked() {
        let mut system_state: SystemState<(
            ResMut<Canvas>,
            ResMut<EdgeManager>,
            Res<FileManager>,
            Res<TabManager>,
        )> = SystemState::new(world);
        let (mut canvas, mut edge_manager, file_manager, tab_manager) = system_state.get_mut(world);

        edge_manager.edge_angle_visibility_toggle(&file_manager, &tab_manager, &mut canvas);

        system_state.apply(world);
    }
}
