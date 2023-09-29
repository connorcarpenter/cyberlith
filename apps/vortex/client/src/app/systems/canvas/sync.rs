use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res, ResMut},
    world::{Mut, World},
};

use input::Input;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{Transform, Visibility},
    Assets,
};

use vortex_proto::components::{EdgeAngle, FileExtension, Vertex3d};

use crate::app::{
    components::{Edge2dLocal, Edge3dLocal, FaceIcon2d, LocalShape},
    resources::{
        animation_manager::{AnimationManager, get_root_vertex}, camera_manager::CameraManager, canvas::Canvas,
        compass::Compass, edge_manager::EdgeManager, face_manager::FaceManager,
        file_manager::FileManager, input_manager::InputManager, tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
};

pub fn queue_resyncs(
    mut canvas: ResMut<Canvas>,
    tab_manager: Res<TabManager>,
    camera_manager: Res<CameraManager>,
    mut compass: ResMut<Compass>,
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
        vertex_manager.queue_resync();
        edge_manager.queue_resync();
        face_manager.queue_resync();
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
    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };
    let camera_state = &current_tab_state.camera_state;
    let camera_3d = camera_manager.camera_3d_entity().unwrap();

    compass.sync_compass(
        &canvas,
        &camera_3d,
        camera_state,
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
    let current_file_entity = *world
        .get_resource::<TabManager>()
        .unwrap()
        .current_tab_entity()
        .unwrap();
    let file_extension = world
        .get_resource::<FileManager>()
        .unwrap()
        .get_file_type(&current_file_entity);

    let camera_3d = world
        .get_resource::<CameraManager>()
        .unwrap()
        .camera_3d_entity()
        .unwrap();
    let camera_state = &current_tab_state.camera_state;
    let camera_3d_scale = camera_state.camera_3d_scale();

    world.resource_scope(|world, mut vertex_manager: Mut<VertexManager>| {
        let should_sync = vertex_manager.should_sync();
        if !should_sync {
            return;
        }
        match file_extension {
            FileExtension::Skel | FileExtension::Mesh => {
                vertex_manager.sync_vertices_3d(world);
                vertex_manager.sync_vertices_2d(world, &camera_3d, camera_3d_scale);
            },
            FileExtension::Anim => {
                let animation_manager = world.get_resource::<AnimationManager>().unwrap();
                if animation_manager.is_posing() {
                    let current_frame_opt = animation_manager.current_frame_entity(&current_file_entity);
                    let root_vertex_opt = get_root_vertex(world);
                    if current_frame_opt.is_some() && root_vertex_opt.is_some() {
                        let frame_entity = current_frame_opt.unwrap();
                        let root_3d_vertex = root_vertex_opt.unwrap();
                        world.resource_scope(|world, animation_manager: Mut<AnimationManager>| {
                            animation_manager.sync_shapes_3d(world, &vertex_manager, camera_3d_scale, frame_entity, root_3d_vertex);
                        });
                    }
                    world.resource_scope(|world, compass: Mut<Compass>| {
                        compass.sync_compass_vertices(world);
                    });
                    vertex_manager.sync_vertices_2d(world, &camera_3d, camera_3d_scale);
                }
            }
            _ => {
                panic!("sync_vertices: unsupported file extension: {:?}", file_extension);
            }
        }

        vertex_manager.finish_resync();
    });
}

pub fn sync_edges(
    file_manager: Res<FileManager>,
    tab_manager: Res<TabManager>,
    canvas: Res<Canvas>,
    mut edge_manager: ResMut<EdgeManager>,
    animation_manager: Res<AnimationManager>,
    local_shape_q: Query<&LocalShape>,
    mut visibility_q: Query<&mut Visibility>,
    mut transform_q: Query<&mut Transform>,
    edge_2d_q: Query<(Entity, &Edge2dLocal)>,
    edge_3d_q: Query<(Entity, &Edge3dLocal, Option<&EdgeAngle>)>,
    edge_angle_q: Query<(Entity, &EdgeAngle)>,
) {
    if !canvas.is_visible() {
        return;
    }
    let Some(current_tab_entity) = tab_manager.current_tab_entity() else {
        return;
    };
    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };
    let file_ext = file_manager.get_file_type(current_tab_entity);

    let camera_state = &current_tab_state.camera_state;
    let camera_3d_scale = camera_state.camera_3d_scale();

    let should_sync = edge_manager.get_should_sync();
    if !should_sync {
        return;
    }

    let should_sync_3d = match file_ext {
        FileExtension::Skel | FileExtension::Mesh => true,
        FileExtension::Anim => animation_manager.current_frame_entity(current_tab_entity).is_none(),
        _ => false,
    };
    if should_sync_3d {
        // animation manager will not handle this, so edge_manager must
        EdgeManager::sync_3d_edges(&edge_3d_q, &mut transform_q, &mut visibility_q);
        edge_manager.sync_edge_angles(
            file_ext,
            &edge_angle_q,
            &mut transform_q,
            &mut visibility_q,
            camera_3d_scale,
        );
    }

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

    edge_manager.finish_sync();
}

pub fn sync_faces(
    tab_manager: Res<TabManager>,
    canvas: Res<Canvas>,
    mut face_manager: ResMut<FaceManager>,
    mut transform_q: Query<&mut Transform>,
    visibility_q: Query<&Visibility>,
    face_2d_q: Query<(Entity, &FaceIcon2d)>,
) {
    if !canvas.is_visible() {
        return;
    }
    let Some(current_tab_state) = tab_manager.current_tab_state() else {
        return;
    };
    let camera_state = &current_tab_state.camera_state;
    let camera_3d_scale = camera_state.camera_3d_scale();

    face_manager.sync_2d_faces(&face_2d_q, &mut transform_q, &visibility_q, camera_3d_scale);
}

pub fn process_faces(
    mut commands: Commands,
    mut canvas: ResMut<Canvas>,
    camera_manager: Res<CameraManager>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
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
}

pub fn update_select_line(
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
        &canvas,
        &file_manager,
        &tab_manager,
        &mut transform_q,
        &mut visibility_q,
        input.mouse_position(),
    );
}
