use bevy_ecs::{
    component::Component,
    entity::Entity,
    event::EventWriter,
    prelude::{Local, With},
    system::{Commands, Query, Res, ResMut},
};

use game_engine::{
    asset::{
        embedded_asset_event, AssetHandle, AssetId, AssetManager, EmbeddedAssetEvent, IconData,
    },
    math::Vec3,
    render::{
        base::{Color, CpuMaterial, CpuMesh},
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
            OrthographicProjection, PerspectiveProjection, PointLight, Projection, RenderLayer,
            RenderLayers, RenderObjectBundle, RenderTarget, Transform, Viewport, Visibility,
        },
        resources::{RenderFrame, Time},
        shapes, Window,
    },
    storage::{Handle, Storage},
    ui::{Alignment, Ui},
};

use crate::app::resources::Global;

#[derive(Component)]
pub struct CubeMarker;

pub fn scene_setup(
    mut commands: Commands,
    mut meshes: ResMut<Storage<CpuMesh>>,
    mut materials: ResMut<Storage<CpuMaterial>>,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
) {
    // TODO: use some kind of catalog here
    embedded_asset_events.send(embedded_asset_event!("../embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("../embedded/34mvvk")); // verdana icon

    let scene_camera = setup_3d_scene(&mut commands, &mut meshes, &mut materials);
    let ui_camera = setup_ui(&mut commands);

    commands.insert_resource(Global::new(scene_camera, ui_camera));
}

fn setup_ui(commands: &mut Commands) -> Entity {
    // render_layer
    let layer = RenderLayers::layer(1);

    // ambient light
    commands
        .spawn(AmbientLight::new(1.0, Color::WHITE))
        .insert(layer);

    // camera
    let camera_id = commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: None, // this will set later
                target: RenderTarget::Screen,
                clear_operation: ClearOperation::none(),
                ..Default::default()
            },
            projection: Projection::Orthographic(OrthographicProjection {
                near: 0.0,
                far: 2000.0,
            }),
            ..Default::default()
        })
        .insert(layer)
        .id();

    // ui

    let text_handle = AssetHandle::<IconData>::new(AssetId::from_str("34mvvk").unwrap()); // TODO: use some kind of catalog

    let mut ui = Ui::new();

    let window_style = ui.create_panel_style(|s| {
        s
            //.set_background_color(Color::BLACK)
            .set_background_alpha(0.0)
            .set_padding_px(10.0, 10.0, 10.0, 10.0)
            .set_vertical()
            .set_row_between_px(10.0);
    });
    let container_style = ui.create_panel_style(|s| {
        s
            .set_background_alpha(0.0)
            .set_size_pc(100.0, 38.2)
            .set_solid_fit()
            .set_aspect_ratio(16.0, 4.5);
    });
    let base_button_style = ui.create_panel_style(|s| {
        s
            .set_background_color(Color::DARK_GRAY)
            .set_self_halign(Alignment::Center)
            .set_size_pc(50.0, 20.0)
            .set_size_max_px(240.0, 90.0)
            .set_solid_fit()
            .set_aspect_ratio(16.0, 4.5)
            .set_padding_px(10.0, 10.0, 10.0, 10.0);
    });
    let start_button_style = ui.create_panel_style(|s| {
       s.set_margin_right_px(40.0);
    });
    let continue_button_style = ui.create_panel_style(|s| {
       s.set_margin_left_px(40.0);
    });

    ui
        .set_text_icon_handle(&text_handle)
        .set_text_color(Color::WHITE)
        .root_mut()
        .add_style(window_style)
        .contents(|c| {
            // title container
            c
                .add_panel()
                .add_style(container_style)
                .contents(|c| {
                    c.add_text("c y b e r l i t h");
                });

            // start button
            c
                .add_panel()
                .add_style(base_button_style)
                .add_style(start_button_style)
                .contents(|c| {
                    c.add_text("start");
                });

            // continue button
            c
                .add_panel()
                .add_style(base_button_style)
                .add_style(continue_button_style)
                .contents(|c| {
                    c.add_text("continue");
                });
        });

    let _ui_entity = commands.spawn(ui).insert(layer).id();

    camera_id
}

fn setup_3d_scene(
    commands: &mut Commands,
    meshes: &mut Storage<CpuMesh>,
    materials: &mut Storage<CpuMaterial>,
) -> Entity {
    // render_layer
    let layer = RenderLayers::layer(0);

    // cube
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(shapes::Cube),
            material: materials.add(Color::RED),
            transform: Transform::from_translation(Vec3::new(0.0, 0.0, 120.0))
                .with_scale(Vec3::splat(30.0)),
            ..Default::default()
        })
        .insert(CubeMarker)
        .insert(layer);

    // ambient light
    commands
        .spawn(AmbientLight::new(0.1, Color::WHITE))
        .insert(layer);

    // directional light
    let light_source = Vec3::new(-500.0, 500.0, 200.0);
    let light_target = Vec3::ZERO;
    commands
        .spawn(DirectionalLight::new(
            2.0,
            Color::WHITE,
            light_target - light_source,
        ))
        .insert(layer);

    // camera
    let camera_id = commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: None,
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Screen,
                ..Default::default()
            },
            transform: Transform::from_xyz(400.0, 400.0, 400.0).looking_at(Vec3::ZERO, Vec3::Z),
            projection:
            // Projection::Orthographic(OrthographicProjection {
            //     near: 0.1,
            //     far: 10000.0,
            //     ..Default::default()
            // }),
                Projection::Perspective(PerspectiveProjection {
                            fov: std::f32::consts::PI / 4.0,
                            near: 0.1,
                            far: 10000.0,
                           }),
        })
        .insert(layer)
        .id();

    camera_id
}

pub fn scene_step(
    time: Res<Time>,
    mut object_q: Query<&mut Transform, With<CubeMarker>>,
    mut rotation: Local<f32>,
) {
    let elapsed_time = time.get_elapsed();

    if *rotation == 0.0 {
        *rotation = 0.01;
    } else {
        *rotation += 0.03 * elapsed_time;
        if *rotation > 359.0 {
            *rotation = 0.01;
        }
    }

    for mut transform in object_q.iter_mut() {
        // rotate
        transform.translation.x = f32::to_radians(*rotation).cos() * 250.0;
        transform.translation.y = f32::to_radians(*rotation).sin() * 250.0;
        transform.translation.z = 60.0;

        transform.rotate_x(0.001 * elapsed_time);
        transform.rotate_y(0.002 * elapsed_time);
    }
}

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
    // UIs
    mut uis_q: Query<(&mut Ui, Option<&RenderLayer>)>,
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
    for (mut ui, render_layer_opt) in uis_q.iter_mut() {
        ui.draw(&mut render_frame, render_layer_opt, &asset_manager);
    }
}

pub fn handle_viewport_resize(
    global: Res<Global>,
    mut window: ResMut<Window>,
    mut cameras_q: Query<(&mut Camera, &mut Transform)>,
) {
    // sync camera viewport to window
    if !window.did_change() {
        return;
    }
    window.clear_change();
    let Some(window_res) = window.get() else {
        return;
    };

    // resize ui camera
    if let Ok((mut camera, mut transform)) = cameras_q.get_mut(global.camera_ui) {
        let should_change = if let Some(viewport) = camera.viewport.as_mut() {
            *viewport != window_res.logical_size
        } else {
            true
        };
        if should_change {
            let new_viewport = Viewport::new_at_origin(
                window_res.logical_size.width,
                window_res.logical_size.height,
            );
            camera.viewport = Some(new_viewport);

            //info!("resize window detected. new viewport: (x: {:?}, y: {:?}, width: {:?}, height: {:?})", new_viewport.x, new_viewport.y, new_viewport.width, new_viewport.height);

            *transform = Transform::from_xyz(
                new_viewport.width as f32 * 0.5,
                new_viewport.height as f32 * 0.5,
                1000.0,
            )
            .looking_at(
                Vec3::new(
                    new_viewport.width as f32 * 0.5,
                    new_viewport.height as f32 * 0.5,
                    0.0,
                ),
                Vec3::NEG_Y,
            );
        }
    }

    if let Ok((mut camera, _transform)) = cameras_q.get_mut(global.camera_3d) {
        let should_change = if let Some(viewport) = camera.viewport.as_mut() {
            *viewport != window_res.logical_size
        } else {
            true
        };
        if should_change {
            let new_viewport = Viewport::new_at_origin(
                window_res.logical_size.width,
                window_res.logical_size.height,
            );
            camera.viewport = Some(new_viewport);

            //info!("resize window detected. new viewport: (x: {:?}, y: {:?}, width: {:?}, height: {:?})", new_viewport.x, new_viewport.y, new_viewport.width, new_viewport.height);
        }
    }

    // resize scene camera
}
