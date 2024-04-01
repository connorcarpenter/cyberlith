use bevy_ecs::{
    change_detection::{Res, ResMut},
    prelude::Query,
};

use game_engine::{
    asset::AssetManager,
    render::{
        components::{AmbientLight, Camera, Projection, RenderLayer, Transform},
        resources::RenderFrame,
    },
    ui::{UiManager, UiRender},
};

pub fn draw(
    mut render_frame: ResMut<RenderFrame>,
    asset_manager: Res<AssetManager>,
    ui_manager: Res<UiManager>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
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
    ui_manager.draw_ui(&asset_manager, &mut render_frame);
}
