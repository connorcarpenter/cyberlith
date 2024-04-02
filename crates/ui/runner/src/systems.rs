use bevy_ecs::{prelude::Query, event::EventReader, change_detection::{Res, ResMut}};

use asset_loader::AssetManager;
use input::InputEvent;
use render_api::components::{Camera, RenderLayer};
use ui_input::UiInputConverter;

use crate::UiManager;

pub fn ui_update(
    mut ui_manager: ResMut<UiManager>,
    asset_manager: Res<AssetManager>,
    mut input_events: EventReader<InputEvent>,
    // Cameras
    cameras_q: Query<(&Camera, Option<&RenderLayer>)>,
) {
    let Some(ui_handle) = ui_manager.active_ui() else {
        return;
    };
    let ui_render_layer_opt = ui_manager.render_layer();

    // find camera, update viewport
    for (camera, cam_render_layer_opt) in cameras_q.iter() {
        if cam_render_layer_opt == ui_render_layer_opt.as_ref() {
            ui_manager.update_ui_viewport(&asset_manager, camera, &ui_handle);
        }
    }

    // update with inputs
    let mut mouse_position_ctnr = None;
    let mut ui_input_events_ctnr = Vec::new();
    ui_manager.generate_new_inputs(&ui_handle, &mut ui_input_events_ctnr);

    let mut next_inputs = Vec::new();
    for event in input_events.read() {
        next_inputs.push(event.clone());
    }

    if let Some((mouse_position, mut ui_input_events)) = UiInputConverter::convert(next_inputs) {
        mouse_position_ctnr = mouse_position;
        ui_input_events_ctnr.append(&mut ui_input_events);
    }
    ui_manager.update_ui_input(&asset_manager, &ui_handle, mouse_position_ctnr, ui_input_events_ctnr);
}