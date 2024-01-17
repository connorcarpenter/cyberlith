use std::collections::HashSet;
use std::time::Duration;

use game_engine::{
    asset::{AnimationData, AssetManager, ModelData},
    math::{Quat, Vec3},
    render::{
        base::{Color, CpuMaterial, CpuMesh},
        components::{
            AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
            OrthographicProjection, PointLight, Projection, RenderLayer, RenderLayers,
            RenderObjectBundle, RenderTarget, Transform, Viewport, Visibility,
        },
        resources::{RenderFrame, Time, WindowSettings},
        shapes, Assets, Handle,
    },
    EnginePlugin,
    http::{HttpRequest, HttpClient, HttpKey},
    naia::Timer,
};

use bevy_app::{App, Startup, Update};
use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Local, Query, Res, ResMut, Resource},
};
use bevy_log::info;

pub fn build() -> App {
    let mut app = App::default();
    app.add_plugins(EnginePlugin)
        // Add Window Settings Plugin
        .insert_resource(WindowSettings {
            title: "Cyberlith".to_string(),
            max_size: Some((1280, 720)),
            ..Default::default()
        })
        // Systems
        .add_systems(Startup, setup)
        .add_systems(Update, step)
        .add_systems(Update, draw)
        // Http
        .init_resource::<ApiTimer>()
        .add_systems(Update, send_recv_http)
    ;
    app
}

// ApiTimer
#[derive(Resource)]
pub struct ApiTimer(pub Timer);

impl Default for ApiTimer {
    fn default() -> Self {
        Self(Timer::new(Duration::from_millis(1000)))
    }
}

// Http systems
fn send_recv_http(mut timer: ResMut<ApiTimer>, mut http_client: ResMut<HttpClient>, mut key_store: Local<HashSet<HttpKey>>) {
    // send
    if timer.0.ringing() {
        timer.0.reset();

        let key = http_client.send(HttpRequest::get("https://api.ipify.org?format=json"));
        key_store.insert(key);
    }

    // recv
    let mut received_keys = Vec::new();
    for key in key_store.iter() {
        if let Some(result) = http_client.recv(key) {
            match result {
                Ok(response) => {
                    let Some(text) = response.text() else {
                        panic!("no text in response");
                    };
                    info!("response: {:?}", text);
                }
                Err(error) => {
                    info!("error: {:?}", error);
                }
            }

            received_keys.push(*key);
        }
    }


    // recv all
    for (key, result) in http_client.recv_all() {
        match result {
            Ok(response) => {
                let Some(text) = response.text() else {
                    panic!("no text in response");
                };
                info!("uncaught response: {:?}", text);
            }
            Err(error) => {
                info!("uncaught error: {:?}", error);

            }
        }

        received_keys.push(key);
    }

    // remove received keys from list
    for key in received_keys {
        key_store.remove(&key);
    }
}

#[derive(Component)]
pub struct ObjectMarker;

const ROOM_WIDTH: f32 = 300.0;
const ROOM_DEPTH: f32 = 300.0;
// const ROOM_HEIGHT: f32 = 200.0;

#[derive(Component)]
pub struct WalkAnimation {
    anim_handle: Handle<AnimationData>,
    image_index: f32,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut asset_manager: ResMut<AssetManager>,
) {
    let layer = RenderLayers::layer(0);

    //let red_mat_handle = materials.add(CpuMaterial::new(Color::RED, 0.0, 32.0, 0.5));

    //let cube_mesh_handle: Handle<MeshFile> = asset_manager.load("cube.mesh");
    // let human_skel_handle = asset_manager.load("human.skel");
    // let threebit_palette_handle = asset_manager.load("3bit.palette");
    // let human_walk_anim_handle: Handle<AnimationData> = asset_manager.load("human_walk.anim");
    //let letters_icon_handle: Handle<IconData> = asset_manager.load("letters.icon");
    // let head_skin_handle: Handle<SkinData> = asset_manager.load("head.skin");
    // let human_model_handle: Handle<ModelData> = asset_manager.load("human.model");
    //let head_scene_handle: Handle<SceneData> = asset_manager.load("head.scene");

    // model
    commands
        .spawn_empty()
        // .insert(human_model_handle)
        // .insert(WalkAnimation {
        //     anim_handle: human_walk_anim_handle,
        //     image_index: 0.0,
        // })
        .insert(
            Transform::from_scale(Vec3::splat(1.0))
                .with_translation(Vec3::splat(0.0))
                .with_rotation(Quat::from_rotation_z(f32::to_radians(0.0))),
        )
        .insert(Visibility::default())
        .insert(ObjectMarker)
        .insert(layer);

    // plane
    commands
        .spawn(RenderObjectBundle {
            mesh: meshes.add(shapes::Square),
            material: materials.add(CpuMaterial::new(Color::DARK_GRAY, 0.0, 0.0, 0.0)),
            transform: Transform::from_scale(Vec3::new(ROOM_WIDTH, ROOM_DEPTH, 1.0))
                .with_translation(Vec3::new(0.0, 0.0, 45.0)),
            ..Default::default()
        })
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
    commands
        .spawn(CameraBundle {
            camera: Camera {
                viewport: Some(Viewport::new_at_origin(1280, 720)),
                order: 0,
                clear_operation: ClearOperation::from_rgba(0.0, 0.0, 0.0, 1.0),
                target: RenderTarget::Screen,
                ..Default::default()
            },
            transform: Transform::from_xyz(400.0, 400.0, 400.0).looking_at(Vec3::ZERO, Vec3::Z),
            projection: Projection::Orthographic(OrthographicProjection {
                near: 0.1,
                far: 10000.0,
                ..Default::default()
            }),
            //     Projection::Perspective(PerspectiveProjection {
            //                 fov: std::f32::consts::PI / 4.0,
            //                 near: 0.1,
            //                 far: 10000.0,
            //                }),
        })
        .insert(layer);
}

fn step(
    time: Res<Time>,
    asset_manager: Res<AssetManager>,
    mut object_q: Query<&mut Transform, With<ObjectMarker>>,
    mut rotation: Local<f32>,
    mut icon_q: Query<&mut WalkAnimation>,
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
        transform.translation.x = f32::to_radians(*rotation).cos() * 250.0;
        transform.translation.y = f32::to_radians(*rotation).sin() * 250.0;
        transform.translation.z = 50.0;

        //transform.rotate_x(0.01 * elapsed_time);
        transform.rotation = Quat::from_rotation_z(f32::to_radians(*rotation + 90.0));
    }

    for mut anim in icon_q.iter_mut() {
        anim.image_index += 0.4 * elapsed_time;

        let subimage_count = asset_manager.get_animation_duration(&anim.anim_handle) as f32;

        while anim.image_index >= subimage_count {
            anim.image_index -= subimage_count;
        }
    }
}

pub fn draw(
    asset_manager: Res<AssetManager>,
    mut render_frame: ResMut<RenderFrame>,
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
    models_q: Query<(
        &Handle<ModelData>,
        &WalkAnimation,
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

    // Aggregate Models
    for (model_handle, walk_anim, transform, visibility, render_layer_opt) in models_q.iter() {
        if !visibility.visible {
            continue;
        }
        asset_manager.draw_animated_model(
            &mut render_frame,
            model_handle,
            &walk_anim.anim_handle,
            transform,
            walk_anim.image_index,
            render_layer_opt,
        );
    }
}
