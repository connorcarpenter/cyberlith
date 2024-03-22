use bevy_ecs::{
    change_detection::{Res, ResMut},
    prelude::Query,
};

use game_engine::{
    asset::{AssetHandle, AssetManager, UiData},
    render::{
        components::{AmbientLight, Camera, Projection, RenderLayer, Transform},
        resources::RenderFrame,
    },
};

pub fn draw(
    mut render_frame: ResMut<RenderFrame>,
    asset_manager: Res<AssetManager>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
    // UIs
    uis_q: Query<(&AssetHandle<UiData>, Option<&RenderLayer>)>,
    // Lights
    ambient_lights_q: Query<(&AmbientLight, Option<&RenderLayer>)>,
) {
    // Aggregate Cameras
    for (camera, transform, projection, render_layer_opt) in cameras_q.iter() {
        if !camera.is_active {
            continue;
        }
        render_frame.draw_camera(render_layer_opt, camera, transform, projection);
    }

    // Aggregate Ambient Lights
    for (ambient_light, render_layer_opt) in ambient_lights_q.iter() {
        render_frame.draw_ambient_light(render_layer_opt, ambient_light);
    }

    // Aggregate UIs
    for (ui_handle, render_layer_opt) in uis_q.iter() {
        asset_manager.draw_ui(&mut render_frame, render_layer_opt, ui_handle);
    }
}