use bevy_ecs::{
    component::Component,
    event::EventWriter,
    system::{Commands, Query, ResMut},
};
use bevy_ecs::system::Res;

use game_engine::{
    asset::{
        embedded_asset_event, EmbeddedAssetEvent,
    },
    math::Vec3,
    render::{
        base::Color,
        components::{
            AmbientLight, Camera, CameraBundle, DirectionalLight, PointLight, Projection,
            RenderLayer, RenderLayers, RenderTarget, Transform, Viewport, OrthographicProjection,
        },
        resources::RenderFrame,
        Window,
    },
    ui::Ui,
};
use game_engine::asset::{AssetHandle, AssetId, AssetManager, IconData};

#[derive(Component)]
pub struct TextMarker;

pub fn scene_setup(
    mut commands: Commands,
    mut embedded_asset_events: EventWriter<EmbeddedAssetEvent>,
) {
    // TODO: use some kind of catalog here
    embedded_asset_events.send(embedded_asset_event!("../embedded/8273wa")); // palette
    embedded_asset_events.send(embedded_asset_event!("../embedded/34mvvk")); // verdana icon

    let layer = RenderLayers::layer(0);

    // ui

    let text_handle = AssetHandle::<IconData>::new(AssetId::from_str("34mvvk").unwrap()); // TODO: use some kind of catalog
    let mut ui = Ui::new();
    ui
        .set_text_handle(& text_handle)
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
            c.add_panel()
                .style(|s| {
                    s
                        .set_background_color(Color::RED)
                        .set_size_st(1.0, 1.0)
                        .set_padding_px(10.0, 10.0, 10.0, 10.0);
                })
                .contents(|mut c| {
                    c.add_label("Hello, my Nina");
                });
            c.add_panel()
                .style(|s| {
                    s
                        .set_background_color(Color::BLUE)
                        .set_size_st(1.0, 1.0);
                })
                .contents(|mut _c| {});
            //ui.button("click me");
        });

    let _ui_entity = commands.spawn(ui).insert(layer).id();

    // ambient light
    commands
        .spawn(AmbientLight::new(1.0, Color::WHITE))
        .insert(layer);

    // camera
    let _camera_id = commands.spawn(CameraBundle {
        camera: Camera {
            viewport: None, // this will set later
            target: RenderTarget::Screen,
            ..Default::default()
        },
        projection: Projection::Orthographic(OrthographicProjection {
            near: 0.0,
            far: 2000.0,
        }),
        ..Default::default()
    }).insert(layer).id();

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
    mut uis_q: Query<(&mut Ui, Option<&RenderLayer>)>,
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

    // Aggregate UIs
    for (mut ui, render_layer_opt) in uis_q.iter_mut() {
        ui.draw(&mut render_frame, render_layer_opt, &asset_manager);
    }
}

pub fn handle_viewport_resize(mut window: ResMut<Window>, mut cameras_q: Query<(&mut Camera, &mut Transform)>) {
    // sync camera viewport to window
    if !window.did_change() {
        return;
    }
    window.clear_change();
    let Some(window_res) = window.get() else {
        return;
    };
    for (mut camera, mut transform) in cameras_q.iter_mut() {
        let should_change = if let Some(viewport) = camera.viewport.as_mut() {
            *viewport != window_res.logical_size } else { true };
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
}