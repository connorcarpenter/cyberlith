use bevy_ecs::{
    change_detection::{Res, ResMut},
    prelude::Query,
};

use game_engine::{
    asset::{AssetHandle, AssetManager, UiData},
    input::winit::{WinitInput, MouseButton},
    render::{
        components::{AmbientLight, Camera, Projection, RenderLayer, Transform},
        resources::RenderFrame,
    },
};

pub fn scene_draw(
    mut render_frame: ResMut<RenderFrame>,
    mut asset_manager: ResMut<AssetManager>,
    input: Res<WinitInput>,
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
    let mouse_pos = input.mouse_position();
    let mouse_state = (
        mouse_pos.x,
        mouse_pos.y,
        input.is_pressed(MouseButton::Left),
    );
    for (ui_handle, render_layer_opt) in uis_q.iter() {
        asset_manager.update_ui(&render_frame, render_layer_opt, mouse_state, ui_handle);
        asset_manager.draw_ui(&mut render_frame, render_layer_opt, ui_handle);
    }
}
