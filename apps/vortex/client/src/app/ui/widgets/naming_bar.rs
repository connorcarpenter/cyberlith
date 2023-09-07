use bevy_ecs::world::World;

use render_egui::{egui, egui::Ui};

use crate::app::{resources::{canvas::Canvas, shape_manager::ShapeManager, toolbar::{Toolbar, ToolbarKind}}, ui::UiState};

pub struct NamingBarWidget {

}

pub fn render_naming_bar(ui: &mut Ui, _world: &mut World) {
    egui::TopBottomPanel::top("naming_bar").show_inside(ui, |_ui| {

    });
}

pub fn naming_bar_visibility_toggle(world: &mut World) {

    // is skeleton toolbar open?
    let toolbar = world.get_resource::<Toolbar>().unwrap();
    let toolbar_kind = toolbar.kind();
    if toolbar_kind != Some(ToolbarKind::Skeleton) {
        return;
    }

    // is vertex/edge selected?
    let shape_manager = world.get_resource::<ShapeManager>().unwrap();
    let selected_shape_2d = shape_manager.selected_shape_2d();
    if selected_shape_2d.is_none() {
        return;
    }

    // actually toggle
    let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
    let old_visible = ui_state.naming_bar_visible;
    ui_state.naming_bar_visible = !old_visible;
    ui_state.resized_window = true;

    // set focus to canvas
    let mut canvas = world.get_resource_mut::<Canvas>().unwrap();
    canvas.set_focused_timed();
}