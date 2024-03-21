use bevy_ecs::system::{Query, Res, ResMut};

use game_engine::{
    asset::{AssetHandle, AssetManager, UiData},
    input::{Input, MouseButton},
    render::{
        base::{CpuMaterial, CpuMesh},
        components::{
            AmbientLight, Camera, DirectionalLight, PointLight, Projection, RenderLayer, Transform,
            Visibility,
        },
        resources::RenderFrame,
    },
    storage::Handle,
};

pub fn draw(
    mut render_frame: ResMut<RenderFrame>,
    mut asset_manager: ResMut<AssetManager>,
    input: Res<Input>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
    // UIs
    uis_q: Query<(&AssetHandle<UiData>, Option<&RenderLayer>)>,
    // Meshes
    cpu_meshes_q: Query<(
        &Handle<CpuMesh>,
        &Handle<CpuMaterial>,
        &Transform,
        &Visibility,
        Option<&RenderLayer>,
    )>,
    // Lights
    ambient_lights_q: Query<(&AmbientLight, Option<&RenderLayer>)>,
    point_lights_q: Query<(&PointLight, Option<&RenderLayer>)>,
    directional_lights_q: Query<(&DirectionalLight, Option<&RenderLayer>)>,
) {
    // Aggregate Cameras
    for (camera, transform, projection, render_layer_opt) in cameras_q.iter() {
        if !camera.is_active {
            continue;
        }
        render_frame.draw_camera(render_layer_opt, camera, transform, projection);
    }

    // Aggregate Point Lights
    for (point_light, render_layer_opt) in point_lights_q.iter() {
        render_frame.draw_point_light(render_layer_opt, point_light);
    }

    // Aggregate Directional Lights
    for (dir_light, render_layer_opt) in directional_lights_q.iter() {
        render_frame.draw_directional_light(render_layer_opt, dir_light);
    }

    // Aggregate Ambient Lights
    for (ambient_light, render_layer_opt) in ambient_lights_q.iter() {
        render_frame.draw_ambient_light(render_layer_opt, ambient_light);
    }

    // Aggregate Cpu Meshes
    for (mesh_handle, mat_handle, transform, visibility, render_layer_opt) in cpu_meshes_q.iter() {
        if !visibility.visible {
            continue;
        }
        render_frame.draw_mesh(render_layer_opt, mesh_handle, mat_handle, transform);
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
