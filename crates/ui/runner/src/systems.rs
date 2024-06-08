use bevy_ecs::{
    change_detection::{Res, ResMut},
    event::EventReader,
    prelude::Query,
};

use asset_loader::AssetManager;
use input::InputEvent;
use render_api::{
    components::{Camera, RenderLayer},
    resources::Time,
};
use ui_input::UiInputConverter;

use crate::UiManager;

pub fn ui_update(
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    time: Res<Time>,
    mut input_events: EventReader<InputEvent>,
    // Cameras
    cameras_q: Query<(&Camera, Option<&RenderLayer>)>,
) {
    ui_manager.update_ui_state(time.get_elapsed_ms());

    if ui_manager.active_ui().is_none() {
        return;
    }
    let ui_render_layer_opt = ui_manager.target_render_layer();

    // find camera, update viewport
    for (target_camera, target_render_layer_opt) in cameras_q.iter() {
        if target_render_layer_opt == ui_render_layer_opt.as_ref() {
            ui_manager.update_ui_viewport(target_camera);
            ui_manager.recalculate_ui_layout_if_needed(&asset_manager);
        }
    }

    // update with inputs
    let mut mouse_position_ctnr = None;
    let mut ui_input_events_ctnr = Vec::new();
    ui_manager.generate_new_inputs(&mut ui_input_events_ctnr);

    let mut next_inputs = Vec::new();
    for event in input_events.read() {
        next_inputs.push(event.clone());
    }

    if let Some((mouse_position, mut ui_input_events)) = UiInputConverter::convert(next_inputs) {
        mouse_position_ctnr = mouse_position;
        ui_input_events_ctnr.append(&mut ui_input_events);
    }
    ui_manager.update_ui_input(&asset_manager, mouse_position_ctnr, ui_input_events_ctnr);
}
