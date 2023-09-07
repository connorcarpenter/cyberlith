use bevy_ecs::{system::Resource, world::World};

use render_egui::{egui, egui::{Align, Ui, Button, Frame, Layout}};

use crate::app::{resources::{canvas::Canvas, shape_manager::ShapeManager, toolbar::{Toolbar, ToolbarKind}}, ui::UiState};

#[derive(Resource)]
pub struct NamingBarState {
    pub(crate) visible: bool,
    prev_text: String,
    text: String,
}

impl Default for NamingBarState {
    fn default() -> Self {
        Self {
            visible: false,
            prev_text: "".to_string(),
            text: "".to_string(),
        }
    }
}

pub fn render_naming_bar(ui: &mut Ui, world: &mut World) {
    egui::TopBottomPanel::top("naming_bar")
        .frame(Frame::central_panel(ui.style()).inner_margin(2.0))
        .show_inside(ui, |ui| {
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {

            let mut state = world.get_resource_mut::<NamingBarState>().unwrap();
            let has_changed = state.prev_text != state.text;

            let button = Button::new("✖").min_size(egui::Vec2::splat(18.0));
            ui.add_enabled(has_changed, button).on_hover_text("Cancel");

            let button = Button::new("✔").min_size(egui::Vec2::splat(18.0));
            ui.add_enabled(has_changed, button).on_hover_text("Accept");

            let text_edit_response = ui.text_edit_singleline(&mut state.text);

            ui.label("name: ");
        });
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
    let mut ui_state = world.get_resource_mut::<NamingBarState>().unwrap();
    let old_visible = ui_state.visible;
    ui_state.visible = !old_visible;

    let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
    ui_state.resized_window = true;

    // set focus to canvas
    let mut canvas = world.get_resource_mut::<Canvas>().unwrap();
    canvas.set_focused_timed();
}