use bevy_ecs::{
    system::{Commands, Query, Res, ResMut, SystemState},
    world::World,
};
use bevy_log::info;

use input::{Input, Key};
use math::Vec3;
use render_api::{
    Assets,
    base::CpuTexture2D,
    components::{Camera, OrthographicProjection, Projection, Transform, Viewport},
};
use render_egui::{
    egui,
    egui::{Frame, Id, Image, pos2, Rect, Ui},
    EguiUserTextures,
};

use crate::app::{plugin::CanvasTexture, resources::global::Global, ui::UiState};

pub fn show_canvas(ui: &mut Ui, world: &mut World) {
    egui::CentralPanel::default()
        .frame(Frame::central_panel(ui.style()).inner_margin(0.0))
        .show_inside(ui, |ui| {
            work_panel(ui, world);
        });
}

fn enable_cameras(global: &Global, camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>, enable_2d: bool, enable_3d: bool) {
    if let Some(camera_2d) = global.camera_2d {
        if let Ok((mut camera, _, _)) = camera_query.get_mut(camera_2d) {
            camera.is_active = enable_2d;
        };
    }
    if let Some(camera_3d) = global.camera_3d {
        if let Ok((mut camera, _, _)) = camera_query.get_mut(camera_3d) {
            camera.is_active = enable_3d;
        };
    }
}

fn enable_3d_shapes(global: &Global, commands: &mut Commands, enable_cube: bool, enable_vertices: bool) {
    if let Some(cube) = global.main_cube {
        if enable_cube {
            commands.entity(cube).insert(global.layer_3d);
        } else {
            commands.entity(cube).insert(global.layer_norender);
        }
    }
    for vertex_3d in global.vertices_3d.iter() {
        if enable_vertices {
            commands.entity(*vertex_3d).insert(global.layer_3d);
        } else {
            commands.entity(*vertex_3d).insert(global.layer_norender);
        }
    }
}

fn work_panel(ui: &mut Ui, world: &mut World) {
    let did_resize = resize_finished(ui, world, "left_panel");

    let mut system_state: SystemState<(
        Commands,
        Res<Global>,
        ResMut<Assets<CpuTexture2D>>,
        ResMut<EguiUserTextures>,
        Res<CanvasTexture>,
        ResMut<UiState>,
        ResMut<Input>,
        Query<(&mut Camera, &mut Transform, &mut Projection)>,
    )> = SystemState::new(world);
    let (
        mut commands,
        global,
        mut textures,
        mut user_textures,
        canvas_texture,
        mut ui_state,
        mut input,
        mut camera_query,
    ) = system_state.get_mut(world);

    // check input
    if input.is_pressed(Key::Q) {

        // disable 2d camera, enable 3d camera
        enable_cameras(&global, &mut camera_query, false, true);

        // disable 3d vertices, enable main cube
        enable_3d_shapes(&global, &mut commands, true, false);
    } else if input.is_pressed(Key::W) {

        // disable 3d camera, enable 2d camera
        enable_cameras(&global, &mut camera_query, true, false);
    } else if input.is_pressed(Key::E) {

        // disable 2d camera, enable 3d camera
        enable_cameras(&global, &mut camera_query, false, true);

        // disable main cube, enable 3d vertices
        enable_3d_shapes(&global, &mut commands, false, true);
    }

    // change textures
    let texture_handle = canvas_texture.0;
    let Some(texture_id) = user_textures.texture_id(&texture_handle) else {
        // The user texture may not be synced yet, return early.
        return;
    };
    let top_left = ui.min_rect().min;
    if ui_state.canvas_coords.is_none() {
        ui_state.canvas_coords = Some(top_left);
        input.set_mouse_offset(top_left.x, top_left.y);
    }
    let texture_size = ui.available_size();
    let image =
        Image::new(texture_id, texture_size).uv(Rect::from_min_max(pos2(0.0, 1.0), pos2(1.0, 0.0)));
    ui.add(image);

    if did_resize {
        ui_state.canvas_coords = Some(top_left);
        input.set_mouse_offset(top_left.x + 1.0, top_left.y + 1.0);

        // This is the texture that will be rendered to.
        let texture_width = texture_size.x as u32;
        let texture_height = texture_size.y as u32;
        let new_texture = CpuTexture2D::from_size(texture_width, texture_height);

        textures.set(&texture_handle, new_texture);
        user_textures.mark_texture_changed(&texture_handle);

        // Update the camera to match the new texture size.
        update_2d_camera(&global, texture_width, texture_height, &mut camera_query);
        update_3d_camera(&global, texture_width, texture_height, &mut camera_query);
    }

    system_state.apply(world);
}

fn update_2d_camera(
    global: &Global,
    texture_width: u32,
    texture_height: u32,
    camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
) {
    let Some(camera_entity) = global.camera_2d else {
        return;
    };
    let Ok((mut camera, mut transform, mut projection)) = camera_query.get_mut(camera_entity) else {
        return;
    };
    camera.viewport = Some(Viewport::new_at_origin(texture_width, texture_height));
    *transform = Transform::from_xyz(
        texture_width as f32 * 0.5,
        texture_height as f32 * 0.5,
        -1.0,
    )
        .looking_at(
            Vec3::new(texture_width as f32 * 0.5, texture_height as f32 * 0.5, 0.0),
            Vec3::NEG_Y,
        );
    *projection = Projection::Orthographic(OrthographicProjection {
        height: texture_height as f32,
        near: 0.0,
        far: 10.0,
    });
}

fn update_3d_camera(
    global: &Global,
    texture_width: u32,
    texture_height: u32,
    camera_query: &mut Query<(&mut Camera, &mut Transform, &mut Projection)>,
) {
    let Some(camera_entity) = global.camera_3d else {
        return;
    };
    let Ok((mut camera, _, _)) = camera_query.get_mut(camera_entity) else {
        return;
    };
    camera.viewport = Some(Viewport::new_at_origin(texture_width, texture_height));
}

fn resize_finished(ui: &Ui, world: &mut World, id_impl: impl Into<Id>) -> bool {
    let Some(mut ui_state) = world.get_resource_mut::<UiState>() else {
        return false;
    };

    let id: Id = id_impl.into();
    let resize_id: Id = id.with("__resize");
    let is_resizing = ui.memory(|mem| mem.is_being_dragged(resize_id));

    if ui_state.dragging_side_panel && !is_resizing {
        ui_state.dragging_side_panel = false;
        return true;
    }
    if !ui_state.dragging_side_panel && is_resizing {
        ui_state.dragging_side_panel = true;
    }

    return false;
}
