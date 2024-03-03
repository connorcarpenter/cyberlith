use bevy_ecs::{
    component::Component,
    system::{Commands, Query, Res, ResMut},
    event::EventWriter
};
use bevy_log::info;

use game_engine::{
    asset::{embedded_asset_event, EmbeddedAssetEvent, IconData, TextStyle, AssetManager, AssetHandle},
    render::{
        base::{Color, CpuMaterial, CpuMesh},
        components::{
            AmbientLight, Camera, CameraBundle, DirectionalLight,
            PointLight, Projection, RenderLayer, RenderLayers,
            RenderTarget, Transform, Viewport, Visibility,
        },
        resources::RenderFrame,
    },
    storage::Handle,
};
use game_engine::math::Vec3;
use game_engine::render::Window;
use game_engine::ui::{Ui};

#[derive(Component)]
pub struct TextMarker;

pub fn scene_setup(
    mut commands: Commands,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>
) {
    // TODO: use some kind of catalog here
    embedded_asset_events.send(embedded_asset_event!("../embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("../embedded/34mvvk")); // verdana icon

    let layer = RenderLayers::layer(0);

    // ui

    let mut ui = Ui::new();
    ui
        .root_mut()
        .style(|s| {
            s
                .set_background_color(Color::YELLOW)
                .set_vertical()
                .set_padding_px(10.0, 10.0, 10.0, 10.0)
                .set_row_between_px(10.0);
        })
        .contents(|mut c| {
            //ui.label("Hello, my Nina! <3");
            c
                .add_panel()
                .style(|s| {
                    s
                        .set_background_color(Color::RED)
                        .set_size_st(1.0, 1.0);
                })
                .contents(|mut _c| {

                });
            c
                .add_panel()
                .style(|s| {
                    s
                        .set_background_color(Color::BLUE)
                        .set_size_st(1.0, 1.0);
                })
                .contents(|mut _c| {

                });
            //ui.button("click me");
        });

    let _ui_entity = commands
        .spawn(ui)
        .insert(layer)
        .id();

    // ambient light
    commands
        .spawn(AmbientLight::new(1.0, Color::WHITE))
        .insert(layer);

    // camera
    let viewport_width = 1280.0;
    let viewport_height = 720.0;
    let mut camera_bundle = CameraBundle::new_2d(&Viewport::new_at_origin(
        viewport_width as u32,
        viewport_height as u32,
    ));
    camera_bundle.camera.target = RenderTarget::Screen;
    let _camera_id = commands
        .spawn(camera_bundle)
        .insert(layer)
        .id();

    // commands
    //     .spawn_empty()
    //     .insert(
    //         Transform::from_translation_2d(Vec2::splat(64.0)),
    //     )
    //     .insert(Visibility::default())
    //     .insert(TextMarker)
    //     .insert(TextStyle::new(32.0, 4.0))
    //     .insert(layer)
    //     .insert(); // TODO: use some kind of catalog
}

pub fn scene_update(
    window: Res<Window>,
    mut cameras_q: Query<(&mut Camera, &mut Transform)>,
) {
    // sync camera viewport to window
    let Some(window_res) = window.get() else {
        return;
    };
    for (mut camera, mut transform) in cameras_q.iter_mut() {
        if let Some(viewport) = camera.viewport.as_mut() {
            if *viewport != window_res.logical_size {
                info!("resize window detected. new viewport: (x: {:?}, y: {:?}, width: {:?}, height: {:?})", viewport.x, viewport.y, viewport.width, viewport.height);
                viewport.x = 0;
                viewport.y = 0;
                viewport.width = window_res.logical_size.width;
                viewport.height = window_res.logical_size.height;

                *transform =  Transform::from_xyz(
                    viewport.width as f32 * 0.5,
                    viewport.height as f32 * 0.5,
                    1000.0,
                )
                .looking_at(
                    Vec3::new(
                        viewport.width as f32 * 0.5,
                        viewport.height as f32 * 0.5,
                        0.0,
                    ),
                    Vec3::NEG_Y,
                );
            }
        }
    }
}

// TODO: handle resize events by updating ui size (ui.update_size())
// TODO: handle mouse move events to update cursor (ui.update_cursor())
// TODO: handle keypress events to update focus (ui.navigate(up/down/left/right))
// TODO: ui.receive_click() -> Option<UiId>(); // ui determined by cursor
// TODO: ui.receive_select() -> UiId; // ui determined by focus
// TODO: ui.receive_char() // if textline widget is in focus

pub fn scene_draw(
    mut render_frame: ResMut<RenderFrame>,
    asset_manager: Res<AssetManager>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
    // Meshes
    cpu_meshes_q: Query<(
        &Handle<CpuMesh>,
        &Handle<CpuMaterial>,
        &Transform,
        &Visibility,
        Option<&RenderLayer>,
    )>,
    icons_q: Query<(
        &AssetHandle<IconData>,
        &TextStyle,
        &Transform,
        &Visibility,
        Option<&RenderLayer>,
    )>,
    mut uis_q: Query<(
        &mut Ui,
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
    for (mut ui, render_layer_opt) in uis_q.iter_mut() {
        ui.draw(&mut render_frame, render_layer_opt);
    }

    //  // Aggregate Icons
    // for (icon_handle, style, transform, visibility, render_layer_opt) in icons_q.iter() {
    //     if !visibility.visible {
    //         continue;
    //     }
    //     asset_manager.draw_text(&mut render_frame, icon_handle, &style, &transform.translation, render_layer_opt, "Hello, my Nina! <3");
    // }
}
