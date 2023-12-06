use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{
    component::Component,
    query::With,
    system::{Commands, Local, Query, Res, ResMut},
};

use asset::{MeshFile, AssetManager, AssetHandle, SkinData, SceneData, ModelData, AnimationData, IconData};
use math::{Quat, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{
        AmbientLight, Camera, CameraBundle, ClearOperation, DirectionalLight,
        OrthographicProjection, PerspectiveProjection, PointLight, Projection, RenderLayer,
        RenderLayers, RenderObjectBundle, RenderTarget, Transform, Viewport, Visibility,
    },
    resources::{RenderFrame, Time, WindowSettings},
    shapes, Assets, Handle,
};

#[derive(Component)]
pub struct ObjectMarker;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // Add Window Settings Plugin
            .insert_resource(WindowSettings {
                title: "Cyberlith".to_string(),
                max_size: Some((1280, 720)),
                ..Default::default()
            })
            // Add Naia Client Plugin
            // .add_plugin(NaiaClientPlugin::new(
            //     NaiaClientConfig::default(),
            //     protocol(),
            // ))
            // Startup Systems
            // .add_startup_system(network::init)
            .add_systems(Startup, setup)
            // Receive Client Events
            // .add_systems(
            //     (
            //         network::connect_events,
            //         network::disconnect_events,
            //         network::reject_events,
            //         network::error_events,
            //     )
            //         .in_set(ReceiveEvents),
            // )
            .add_systems(Update, step)
            .add_systems(Update, draw);
    }
}

const ROOM_WIDTH: f32 = 300.0;
const ROOM_DEPTH: f32 = 300.0;
const ROOM_HEIGHT: f32 = 200.0;

#[derive(Component)]
pub struct IconAnimation {
    subimage_index: f32,
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
    let letters_icon_handle: Handle<IconData> = asset_manager.load("letters.icon");
    // let head_skin_handle: Handle<SkinData> = asset_manager.load("head.skin");
    // let human_model_handle: Handle<ModelData> = asset_manager.load("human.model");
    //let head_scene_handle: Handle<SceneData> = asset_manager.load("head.scene");

    // model
    commands
        .spawn_empty()
        .insert(letters_icon_handle)
        .insert(IconAnimation {
            subimage_index: 0.0,
        })
        .insert(Transform::from_scale(Vec3::splat(1.0))
            .with_translation(Vec3::splat(0.0))
            .with_rotation(Quat::from_rotation_z(f32::to_radians(0.0))))
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
            projection:
            Projection::Orthographic(
                OrthographicProjection {
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
    mut icon_q: Query<(&Handle<IconData>, &mut IconAnimation)>,
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

    for (handle, mut anim) in icon_q.iter_mut() {

        anim.subimage_index += (0.32 * elapsed_time);

        let subimage_count = asset_manager.get_icon_subimage_count(handle) as f32;

        while anim.subimage_index >= subimage_count {
            anim.subimage_index -= subimage_count;
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
    icons_q: Query<(
        &Handle<IconData>,
        &IconAnimation,
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

    // Aggregate File Meshes
    for (icon_handle, icon_anim, transform, visibility, render_layer_opt) in icons_q.iter() {
        if !visibility.visible {
            continue;
        }
        asset_manager.draw_icon(&mut render_frame, icon_handle, icon_anim.subimage_index as usize, transform, render_layer_opt);
    }
}
