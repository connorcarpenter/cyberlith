use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    system::{Commands, Query, Res, ResMut, SystemState},
    world::{Mut, World},
};

use naia_bevy_client::Client;

use input::Input;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{Transform, Visibility},
    Assets,
};

use editor_proto::components::{
    AnimFrame, EdgeAngle, FileExtension, IconFrame, PaletteColor, Vertex3d,
};

use crate::app::{
    components::{Edge2dLocal, Edge3dLocal, FaceIcon2d, LocalShape},
    events::ShapeColorResyncEvent,
    plugin::Main,
    resources::{
        animation_manager::{get_root_vertex, AnimationManager},
        camera_manager::CameraManager,
        canvas::Canvas,
        compass::Compass,
        edge_manager::EdgeManager,
        face_manager::FaceManager,
        file_manager::FileManager,
        grid::Grid,
        icon_manager::IconManager,
        input::InputManager,
        model_manager::ModelManager,
        palette_manager::PaletteManager,
        tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
};

pub fn queue_resyncs(
    mut canvas: ResMut<Canvas>,
    tab_manager: Res<TabManager>,
    camera_manager: Res<CameraManager>,
    mut compass: ResMut<Compass>,
    mut grid: ResMut<Grid>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut input_manager: ResMut<InputManager>,
    mut face_manager: ResMut<FaceManager>,
) {
    if !canvas.is_visible() {
        return;
    }
    if tab_manager.current_tab_entity().is_none() {
        return;
    }
    if tab_manager.current_tab_state().is_none() {
        return;
    };
    if camera_manager.camera_3d_entity().is_none() {
        return;
    }

    let should_sync_shapes = canvas.should_sync_shapes();
    if should_sync_shapes {
        input_manager.queue_resync_hover_ui();
        input_manager.queue_resync_selection_ui();
        compass.queue_resync();
        grid.queue_resync();
        vertex_manager.queue_resync();
        edge_manager.queue_resync();
        face_manager.queue_resync();
    }
}

pub fn queue_shape_color_resync(
    mut tab_manager: ResMut<TabManager>,
    mut event_reader: EventReader<ShapeColorResyncEvent>,
) {
    let mut did_receive = false;
    for _event in event_reader.read() {
        did_receive = true;
    }
    if did_receive {
        tab_manager.resync_shape_colors();
    }
}

pub fn sync_compass(
    canvas: Res<Canvas>,
    tab_manager: Res<TabManager>,
    camera_manager: Res<CameraManager>,
    mut compass: ResMut<Compass>,
    transform_q: Query<&Transform>,
    mut vertex_3d_q: Query<(Entity, &mut Vertex3d)>,
) {
    compass.sync_compass(
        &canvas,
        &tab_manager,
        &camera_manager,
        &mut vertex_3d_q,
        &transform_q,
    );
}

pub fn sync_vertices(world: &mut World) {
    if !world.get_resource::<Canvas>().unwrap().is_visible() {
        return;
    }
    let Some(current_tab_state) = world.get_resource::<TabManager>().unwrap().current_tab_state() else {
        return;
    };
    let should_sync = world.get_resource::<VertexManager>().unwrap().should_sync();
    if !should_sync {
        return;
    }
    let current_file_entity = *world
        .get_resource::<TabManager>()
        .unwrap()
        .current_tab_entity()
        .unwrap();
    let file_extension = world
        .get_resource::<FileManager>()
        .unwrap()
        .get_file_type(&current_file_entity);

    let camera_3d_entity = world
        .get_resource::<CameraManager>()
        .unwrap()
        .camera_3d_entity()
        .unwrap();
    let camera_state = &current_tab_state.camera_state;
    let camera_is_2d = camera_state.is_2d();
    let camera_3d_scale = camera_state.camera_3d_scale();

    world.resource_scope(|world, mut vertex_manager: Mut<VertexManager>| {
        match file_extension {
            FileExtension::Skel | FileExtension::Mesh | FileExtension::Skin => {
                vertex_manager.sync_3d_vertices(file_extension, world);
                vertex_manager.sync_2d_vertices(
                    file_extension,
                    world,
                    &camera_3d_entity,
                    camera_3d_scale,
                );
            }
            FileExtension::Model | FileExtension::Scene => {
                world.resource_scope(|world, mut model_manager: Mut<ModelManager>| {
                    model_manager.sync_shapes(
                        world,
                        &vertex_manager,
                        &file_extension,
                        &current_file_entity,
                        &camera_3d_entity,
                        camera_is_2d,
                        camera_3d_scale,
                    );
                });
            }
            FileExtension::Anim => {
                let animation_manager = world.get_resource::<AnimationManager>().unwrap();
                if animation_manager.is_posing() {
                    if let Some(root_3d_vertex) = get_root_vertex(world) {
                        let (frame_entity_opt, interp_opt) = {
                            let animation_manager =
                                world.get_resource::<AnimationManager>().unwrap();
                            if animation_manager.preview_frame_selected() {
                                let mut prev_frame_index = animation_manager.preview_frame_index();
                                let frame_entity = animation_manager
                                    .get_frame_entity(&current_file_entity, prev_frame_index)
                                    .unwrap();
                                let frame_count = animation_manager
                                    .get_frame_count(&current_file_entity)
                                    .unwrap();
                                let elapsed_ms = animation_manager.preview_elapsed_ms();

                                let frame_component = world
                                    .query::<&AnimFrame>()
                                    .get(world, frame_entity)
                                    .unwrap();
                                let frame_duration =
                                    frame_component.transition.get_duration_ms() as f32;

                                prev_frame_index += 1;
                                if prev_frame_index >= frame_count {
                                    prev_frame_index -= frame_count;
                                }

                                let animation_manager =
                                    world.get_resource::<AnimationManager>().unwrap();
                                let next_entity = animation_manager
                                    .get_frame_entity(&current_file_entity, prev_frame_index)
                                    .unwrap();

                                let interp = elapsed_ms / frame_duration;
                                (Some(frame_entity), Some((next_entity, interp)))
                            } else {
                                (
                                    animation_manager.current_frame_entity(&current_file_entity),
                                    None,
                                )
                            }
                        };

                        if frame_entity_opt.is_some() {
                            let frame_entity = frame_entity_opt.unwrap();
                            world.resource_scope(
                                |world, animation_manager: Mut<AnimationManager>| {
                                    animation_manager.sync_shapes_3d(
                                        world,
                                        &vertex_manager,
                                        camera_3d_scale,
                                        frame_entity,
                                        interp_opt,
                                        root_3d_vertex,
                                    );
                                },
                            );
                        }
                    }
                    world.resource_scope(|world, compass: Mut<Compass>| {
                        compass.sync_compass_vertices(world);
                    });
                    world.resource_scope(|world, mut grid: Mut<Grid>| {
                        grid.sync_grid_vertices(world);
                    });
                    vertex_manager.sync_2d_vertices(
                        file_extension,
                        world,
                        &camera_3d_entity,
                        camera_3d_scale,
                    );
                }
            }
            FileExtension::Palette => {
                // skip, no vertices here
            }
            FileExtension::Icon => {
                // no need to sync - everything is 2d
            }
            _ => {
                panic!(
                    "sync_vertices: unsupported file extension: {:?}",
                    file_extension
                );
            }
        }

        vertex_manager.finish_resync();
    });
}

pub fn sync_edges(world: &mut World) {
    let should_sync = world
        .get_resource::<EdgeManager>()
        .unwrap()
        .get_should_sync();
    if !should_sync {
        return;
    }

    if !world.get_resource::<Canvas>().unwrap().is_visible() {
        return;
    }
    let Some(current_tab_entity) = world.get_resource::<TabManager>().unwrap().current_tab_entity() else {
        return;
    };
    let current_tab_entity = *current_tab_entity;

    let file_ext = world
        .get_resource::<FileManager>()
        .unwrap()
        .get_file_type(&current_tab_entity);

    match file_ext {
        FileExtension::Skel | FileExtension::Mesh | FileExtension::Skin => {
            let mut system_state: SystemState<(
                Res<TabManager>,
                ResMut<EdgeManager>,
                Query<&LocalShape>,
                Query<&mut Visibility>,
                Query<&mut Transform>,
                Query<(Entity, &Edge2dLocal)>,
                Query<(Entity, &Edge3dLocal, Option<&EdgeAngle>)>,
                Query<(Entity, &EdgeAngle)>,
            )> = SystemState::new(world);
            let (
                tab_manager,
                mut edge_manager,
                local_shape_q,
                mut visibility_q,
                mut transform_q,
                edge_2d_q,
                edge_3d_q,
                edge_angle_q,
            ) = system_state.get_mut(world);

            let current_tab_state = tab_manager.current_tab_state().unwrap();
            let camera_state = &current_tab_state.camera_state;
            let camera_3d_scale = camera_state.camera_3d_scale();
            sync_3d_edges(
                &mut edge_manager,
                &local_shape_q,
                &mut visibility_q,
                &mut transform_q,
                &edge_3d_q,
                &edge_angle_q,
                file_ext,
                camera_3d_scale,
            );
            sync_2d_edges(
                &mut edge_manager,
                &local_shape_q,
                &mut visibility_q,
                &mut transform_q,
                &edge_2d_q,
                &edge_3d_q,
                camera_3d_scale,
            );
        }
        FileExtension::Anim => {
            let mut system_state: SystemState<(
                Res<TabManager>,
                ResMut<EdgeManager>,
                Res<AnimationManager>,
                Query<&LocalShape>,
                Query<&mut Visibility>,
                Query<&mut Transform>,
                Query<(Entity, &Edge2dLocal)>,
                Query<(Entity, &Edge3dLocal, Option<&EdgeAngle>)>,
                Query<(Entity, &EdgeAngle)>,
            )> = SystemState::new(world);
            let (
                tab_manager,
                mut edge_manager,
                animation_manager,
                local_shape_q,
                mut visibility_q,
                mut transform_q,
                edge_2d_q,
                edge_3d_q,
                edge_angle_q,
            ) = system_state.get_mut(world);

            let current_tab_state = tab_manager.current_tab_state().unwrap();
            let camera_state = &current_tab_state.camera_state;
            let camera_3d_scale = camera_state.camera_3d_scale();
            if animation_manager
                .current_frame_entity(&current_tab_entity)
                .is_none()
            {
                sync_3d_edges(
                    &mut edge_manager,
                    &local_shape_q,
                    &mut visibility_q,
                    &mut transform_q,
                    &edge_3d_q,
                    &edge_angle_q,
                    file_ext,
                    camera_3d_scale,
                );
            }
            sync_2d_edges(
                &mut edge_manager,
                &local_shape_q,
                &mut visibility_q,
                &mut transform_q,
                &edge_2d_q,
                &edge_3d_q,
                camera_3d_scale,
            );
        }
        FileExtension::Model | FileExtension::Scene => {
            // handles in "sync_vertices"
        }
        _ => {}
    };

    world
        .get_resource_mut::<EdgeManager>()
        .unwrap()
        .finish_sync();
}

fn sync_3d_edges(
    edge_manager: &mut EdgeManager,
    local_shape_q: &Query<&LocalShape>,
    mut visibility_q: &mut Query<&mut Visibility>,
    mut transform_q: &mut Query<&mut Transform>,
    edge_3d_q: &Query<(Entity, &Edge3dLocal, Option<&EdgeAngle>)>,
    edge_angle_q: &Query<(Entity, &EdgeAngle)>,
    file_ext: FileExtension,
    camera_3d_scale: f32,
) {
    // animation manager will not handle this, so edge_manager must
    EdgeManager::sync_3d_edges(
        file_ext,
        &edge_3d_q,
        &mut transform_q,
        &mut visibility_q,
        &local_shape_q,
    );
    edge_manager.sync_edge_angles(
        file_ext,
        &edge_angle_q,
        &mut transform_q,
        &mut visibility_q,
        camera_3d_scale,
    );
}

fn sync_2d_edges(
    edge_manager: &mut ResMut<EdgeManager>,
    local_shape_q: &Query<&LocalShape>,
    mut visibility_q: &mut Query<&mut Visibility>,
    mut transform_q: &mut Query<&mut Transform>,
    edge_2d_q: &Query<(Entity, &Edge2dLocal)>,
    edge_3d_q: &Query<(Entity, &Edge3dLocal, Option<&EdgeAngle>)>,
    camera_3d_scale: f32,
) {
    EdgeManager::sync_local_3d_edges(
        &edge_3d_q,
        &mut transform_q,
        &local_shape_q,
        camera_3d_scale,
    );
    edge_manager.sync_2d_edges(
        &edge_2d_q,
        &mut transform_q,
        &mut visibility_q,
        &local_shape_q,
        camera_3d_scale,
    );
}

pub fn sync_faces(
    file_manager: Res<FileManager>,
    tab_manager: Res<TabManager>,
    canvas: Res<Canvas>,
    mut face_manager: ResMut<FaceManager>,
    mut transform_q: Query<&mut Transform>,
    mut visibility_q: Query<&mut Visibility>,
    face_2d_q: Query<(Entity, &FaceIcon2d)>,
) {
    if !canvas.is_visible() {
        return;
    }
    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };
    let Some(current_file_entity) = tab_manager.current_tab_entity() else {
        return;
    };
    let file_ext = file_manager.get_file_type(current_file_entity);

    let camera_state = &current_tab_state.camera_state;
    let camera_3d_scale = camera_state.camera_3d_scale();

    face_manager.sync_2d_faces(
        file_ext,
        &face_2d_q,
        &mut transform_q,
        &mut visibility_q,
        camera_3d_scale,
    );
}

pub fn update_animation(
    client: Client<Main>,
    mut canvas: ResMut<Canvas>,
    file_manager: Res<FileManager>,
    tab_manager: Res<TabManager>,
    mut animation_manager: ResMut<AnimationManager>,
    frame_q: Query<(Entity, &AnimFrame)>,
) {
    // get file type
    let Some(current_file_entity) = tab_manager.current_tab_entity() else {
        return;
    };
    let file_ext = file_manager.get_file_type(current_file_entity);
    if file_ext != FileExtension::Anim {
        return;
    }
    animation_manager.framing_resync_frame_order(&client, &frame_q);
    animation_manager.preview_update(&mut canvas, current_file_entity, &frame_q);
}

pub fn update_icon(
    client: Client<Main>,
    file_manager: Res<FileManager>,
    tab_manager: Res<TabManager>,
    mut icon_manager: ResMut<IconManager>,
    frame_q: Query<(Entity, &IconFrame)>,
) {
    // get file type
    let Some(current_file_entity) = tab_manager.current_tab_entity() else {
        return;
    };
    let file_ext = file_manager.get_file_type(current_file_entity);
    if file_ext != FileExtension::Icon {
        return;
    }
    icon_manager.framing_resync_frame_order(&client, &frame_q);
    icon_manager.preview_update(current_file_entity);
}

pub fn update_palette(
    client: Client<Main>,
    file_manager: Res<FileManager>,
    tab_manager: Res<TabManager>,
    mut palette_manager: ResMut<PaletteManager>,
    color_q: Query<(Entity, &PaletteColor)>,
) {
    // get file type
    let Some(current_file_entity) = tab_manager.current_tab_entity() else {
        return;
    };
    let file_ext = file_manager.get_file_type(current_file_entity);
    if file_ext != FileExtension::Palette {
        return;
    }
    palette_manager.resync_color_order(&client, &color_q);
}

pub fn process_faces(
    mut commands: Commands,
    mut canvas: ResMut<Canvas>,
    camera_manager: Res<CameraManager>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut icon_manager: ResMut<IconManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
) {
    face_manager.process_new_faces(
        &mut commands,
        &mut canvas,
        &camera_manager,
        &mut vertex_manager,
        &mut edge_manager,
        &mut meshes,
        &mut materials,
    );
    icon_manager.process_new_local_faces(&mut commands, &mut meshes, &mut materials);
}

pub fn update_selection_ui(
    canvas: Res<Canvas>,
    file_manager: Res<FileManager>,
    tab_manager: Res<TabManager>,
    input: Res<Input>,
    mut input_manager: ResMut<InputManager>,
    mut transform_q: Query<&mut Transform>,
    mut visibility_q: Query<&mut Visibility>,
) {
    if !canvas.is_visible() {
        return;
    }

    input_manager.sync_selection_ui(
        &file_manager,
        &tab_manager,
        &mut transform_q,
        &mut visibility_q,
        input.mouse_position(),
    );
}
