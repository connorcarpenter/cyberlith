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
    let Some((mouse_position, ui_input_events)) = UiInputConverter::convert(&mut input_events)
        else {
            return;
        };
    ui_manager.update_ui_input(&asset_manager, &ui_handle, mouse_position, ui_input_events);
}