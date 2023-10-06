use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::{EventReader, EventWriter},
    system::{SystemState, Commands, Query, ResMut},
    world::World,
};
use bevy_log::{info, warn};

use naia_bevy_client::{events::InsertComponentEvents, Client, Replicate};

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::Transform,
    Assets,
};

use vortex_proto::components::{AnimFrame, AnimRotation, ChangelistEntry, ChangelistStatus, Edge3d, EdgeAngle, EntryKind, Face3d, FileDependency, FileExtension, FileSystemChild, FileSystemEntry, FileSystemRootChild, FileType, OwnedByFile, PaletteColor, ShapeName, Vertex3d, VertexRoot};

use crate::app::{
    components::file_system::{
        ChangelistUiState, FileSystemEntryLocal, FileSystemParent, FileSystemUiState,
    },
    events::InsertComponentEvent,
    resources::{
        animation_manager::AnimationManager,
        camera_manager::CameraManager,
        canvas::Canvas,
        edge_manager::EdgeManager,
        face_manager::FaceManager,
        file_manager::{get_full_path, FileManager},
        shape_waitlist::{ShapeWaitlist, ShapeWaitlistInsert},
        tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
    systems::file_post_process,
};
use crate::app::resources::palette_manager::PaletteManager;

// if this gets big, just switch to a &mut World
pub fn insert_component_events(
    world: &mut World,
) {
    let mut system_state: SystemState<EventReader<InsertComponentEvents>> = SystemState::new(world);
    let mut events_reader = system_state.get_mut(world);

    let mut events_collection: Vec<InsertComponentEvents> = Vec::new();
    for events in events_reader.iter() {
        events_collection.push(events.clone());
    }

    for events in events_collection {
        insert_component_event::<FileSystemEntry>       (world, &events);
        insert_component_event::<FileSystemRootChild>   (world, &events);
        insert_component_event::<FileSystemChild>       (world, &events);
        insert_component_event::<FileDependency>        (world, &events);
        insert_component_event::<ChangelistEntry>       (world, &events);
        insert_component_event::<Vertex3d>              (world, &events);
        insert_component_event::<VertexRoot>            (world, &events);
        insert_component_event::<ShapeName>             (world, &events);
        insert_component_event::<OwnedByFile>           (world, &events);
        insert_component_event::<FileType>              (world, &events);
        insert_component_event::<Edge3d>                (world, &events);
        insert_component_event::<EdgeAngle>             (world, &events);
        insert_component_event::<Face3d>                (world, &events);
        insert_component_event::<AnimFrame>             (world, &events);
        insert_component_event::<AnimRotation>          (world, &events);
        insert_component_event::<PaletteColor>          (world, &events);
    }
}

fn insert_component_event<T: Replicate>(world: &mut World, events: &InsertComponentEvents) {
    let mut system_state: SystemState<EventWriter<InsertComponentEvent<T>>> = SystemState::new(world);
    let mut event_writer = system_state.get_mut(world);

    for entity in events.read::<T>() {
        event_writer.send(InsertComponentEvent::<T>::new(entity));
    }
}

pub fn insert_file_component_events(
    mut commands: Commands,
    client: Client,
    mut file_manager: ResMut<FileManager>,
    mut tab_manager: ResMut<TabManager>,
    mut entry_events: EventReader<InsertComponentEvent<FileSystemEntry>>,
    mut root_events: EventReader<InsertComponentEvent<FileSystemRootChild>>,
    mut child_events: EventReader<InsertComponentEvent<FileSystemChild>>,
    mut dependency_events: EventReader<InsertComponentEvent<FileDependency>>,
    entry_q: Query<&FileSystemEntry>,
    mut parent_q: Query<&mut FileSystemParent>,
    child_q: Query<&FileSystemChild>,
    dependency_q: Query<&FileDependency>,
) {
    let project_root_entity = file_manager.project_root_entity;
    let mut recent_parents: Option<HashMap<Entity, FileSystemParent>> = None;

    for event in entry_events.iter() {
        let entity = event.entity;
        let entry = entry_q.get(entity).unwrap();

        info!(
            "entity: `{:?}` - inserted FileSystemEntry. kind: `{:?}`, name: `{:?}`",
            entity, *entry.kind, *entry.name
        );

        file_post_process::insert_ui_state_component(&mut commands, entity, false);
        if *entry.kind == EntryKind::Directory {
            if recent_parents.is_none() {
                recent_parents = Some(HashMap::new());
            }
            recent_parents
                .as_mut()
                .unwrap()
                .insert(entity, FileSystemParent::new());
        }
        file_manager.on_file_create(&entity, FileExtension::from(entry.name.as_str()));
        commands
            .entity(entity)
            .insert(FileSystemEntryLocal::new(entry.name.as_str()));
    }

    for event in root_events.iter() {
        let entity = event.entity;
        // Add children to root parent
        let entry = entry_q.get(entity).unwrap();
        let mut parent = parent_q.get_mut(project_root_entity).unwrap();
        file_post_process::parent_add_child_entry(&mut parent, entry, entity);
    }

    for event in child_events.iter() {
        let entity = event.entity;
        let entry = entry_q.get(entity).unwrap();
        // Get parent
        let Some(parent_entity) = child_q
                .get(entity)
                .unwrap()
                .parent_id
                .get(&client) else {
                panic!("FileSystemChild component of entry: `{}` has no parent component", *entry.name);
            };

        if let Ok(mut parent) = parent_q.get_mut(parent_entity) {
            file_post_process::parent_add_child_entry(&mut parent, entry, entity);
        } else {
            let Some(parent_map) = recent_parents.as_mut() else {
                    panic!("FileSystemChild component on entity: `{:?}` has invalid parent_id: `{:?}`", entity, parent_entity);
                };
            let Some(parent) = parent_map.get_mut(&parent_entity) else {
                    panic!("FileSystemChild component on entity: `{:?}` has invalid parent_id: `{:?}`", entity, parent_entity);
                };
            file_post_process::parent_add_child_entry(parent, entry, entity);
        }
    }

    // Add all parents now that the children were able to process
    // Note that we do it this way because Commands aren't flushed till the end of the system
    if let Some(parent_map) = recent_parents.as_mut() {
        for (entity, parent) in parent_map.drain() {
            commands.entity(entity).insert(parent);
        }
    }

    for event in dependency_events.iter() {
        let entity = event.entity;
        let component = dependency_q.get(entity).unwrap();
        let file_entity = component.file_entity.get(&client).unwrap();
        let dependency_entity = component.dependency_entity.get(&client).unwrap();

        file_manager.file_add_dependency(&file_entity, &dependency_entity);

        info!(
            "received FileDependency(file: `{:?}`, dependency: `{:?}`)",
            file_entity, dependency_entity
        );

        tab_manager.resync_tab_ownership();
    }
}

pub fn insert_changelist_entry_events(
    mut commands: Commands,
    mut file_manager: ResMut<FileManager>,
    client: Client,
    mut events: EventReader<InsertComponentEvent<ChangelistEntry>>,
    changelist_q: Query<&ChangelistEntry>,
    fs_q: Query<(&FileSystemEntry, Option<&FileSystemChild>)>,
    mut fs_state_q: Query<&mut FileSystemUiState>,
) {
    // on ChangelistEntry Insert Event
    for event in events.iter() {
        let cl_entity = event.entity;

        let entry = changelist_q.get(cl_entity).unwrap();

        info!(
            "entity: `{:?}` - inserted ChangelistEntry. path: `{:?}`, name: `{:?}`",
            cl_entity, *entry.path, *entry.name
        );

        let mut display_name: String = (*entry.name).clone();
        let mut display_path: String = (*entry.path).clone();
        let mut file_entity_opt: Option<Entity> = None;
        let mut parent_entity_opt: Option<Entity> = None;

        // associate status with file entry
        if *entry.status != ChangelistStatus::Deleted {
            let file_entity = entry.file_entity.get(&client).unwrap();

            let mut fs_state = fs_state_q.get_mut(file_entity).unwrap();
            fs_state.change_status = Some(*entry.status);

            let (fs_entry, fs_child_opt) = fs_q.get(file_entity).unwrap();
            display_name = (*fs_entry.name).clone();
            file_entity_opt = Some(file_entity);

            if let Some(fs_child) = fs_child_opt {
                parent_entity_opt = fs_child.parent_id.get(&client);
            }

            if parent_entity_opt.is_some() {
                let new_display_path = get_full_path(&client, &fs_q, file_entity);
                info!(
                    "change path for entity: `{:?}`. path was: `{:?}`, now is `{:?}`",
                    file_entity, display_path, new_display_path
                );
                display_path = new_display_path;
            }
        }

        // insert ui state component
        commands
            .entity(cl_entity)
            .insert(ChangelistUiState::new(&display_name, &display_path));

        // insert into changelist resource
        file_manager.insert_changelist_entry(
            entry.file_key(),
            file_entity_opt,
            parent_entity_opt,
            cl_entity,
        );
    }
}

pub fn insert_vertex_events(
    mut commands: Commands,
    mut vertex_3d_events: EventReader<InsertComponentEvent<Vertex3d>>,
    mut vertex_root_events: EventReader<InsertComponentEvent<VertexRoot>>,
    mut shape_name_events: EventReader<InsertComponentEvent<ShapeName>>,

    mut camera_manager: ResMut<CameraManager>,
    mut canvas: ResMut<Canvas>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,
    transform_q: Query<&Transform>,
    shape_name_q: Query<&ShapeName>,
) {
    // on Vertex Insert Event
    for event in vertex_3d_events.iter() {
        let entity = event.entity;

        info!("entity: {:?} - inserted Vertex3d", entity);

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &entity,
            ShapeWaitlistInsert::Vertex,
        );
    }

    // on Vertex Root Event
    for event in vertex_root_events.iter() {
        let entity = event.entity;

        info!("entity: {:?} - inserted VertexRoot", entity);

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &entity,
            ShapeWaitlistInsert::VertexRoot,
        );
    }

    // on ShapeName Event
    for event in shape_name_events.iter() {
        let entity = event.entity;

        let shape_name = shape_name_q.get(entity).unwrap();
        let shape_name = (*shape_name.value).clone();

        info!(
            "entity: {:?} - inserted ShapeName(name: {:?})",
            entity, shape_name
        );
    }
}

pub fn insert_edge_events(
    mut commands: Commands,
    client: Client,
    mut edge_3d_events: EventReader<InsertComponentEvent<Edge3d>>,
    mut edge_angle_events: EventReader<InsertComponentEvent<EdgeAngle>>,

    // for vertices
    mut camera_manager: ResMut<CameraManager>,
    mut canvas: ResMut<Canvas>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,

    edge_3d_q: Query<&Edge3d>,
    edge_angle_q: Query<&EdgeAngle>,
    transform_q: Query<&Transform>,
) {
    // on Edge3d Insert Event
    for event in edge_3d_events.iter() {
        // handle vertex
        let edge_entity = event.entity;

        info!("entity: {:?} - inserted Edge3d", edge_entity);

        let edge_3d = edge_3d_q.get(edge_entity).unwrap();
        let Some(start_entity) = edge_3d.start.get(&client) else {
            warn!("Edge3d component of entity: `{:?}` has no start entity", edge_entity);
            continue;
        };
        let Some(end_entity) = edge_3d.end.get(&client) else {
            warn!("Edge3d component of entity: `{:?}` has no start entity", edge_entity);
            continue;
        };

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &edge_entity,
            ShapeWaitlistInsert::Edge(start_entity, end_entity),
        );
    }

    // on EdgeAngle Insert Event
    for event in edge_angle_events.iter() {
        // handle vertex
        let edge_entity = event.entity;

        info!("entity: {:?} - inserted EdgeAngle", edge_entity);

        let edge_3d = edge_angle_q.get(edge_entity).unwrap();

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &edge_entity,
            ShapeWaitlistInsert::EdgeAngle(edge_3d.get_radians()),
        );
    }
}

pub fn insert_face_events(
    mut commands: Commands,
    client: Client,
    mut face_3d_events: EventReader<InsertComponentEvent<Face3d>>,
    mut camera_manager: ResMut<CameraManager>,
    mut canvas: ResMut<Canvas>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,

    face_3d_q: Query<&Face3d>,
    transform_q: Query<&Transform>,
) {
    // on Face3d Insert Event
    for event in face_3d_events.iter() {
        // handle face
        let face_entity = event.entity;

        info!("entity: {:?} - inserted Face3d", face_entity);

        let face_3d = face_3d_q.get(face_entity).unwrap();
        let Some(vertex_a_entity) = face_3d.vertex_a.get(&client) else {
            warn!("Face3d component of entity: `{:?}` has no entity", face_entity);
            continue;
        };
        let Some(vertex_b_entity) = face_3d.vertex_b.get(&client) else {
            warn!("Face3d component of entity: `{:?}` has no entity", face_entity);
            continue;
        };
        let Some(vertex_c_entity) = face_3d.vertex_c.get(&client) else {
            warn!("Face3d component of entity: `{:?}` has no entity", face_entity);
            continue;
        };
        let Some(edge_a_entity) = face_3d.edge_a.get(&client) else {
            warn!("Face3d component of entity: `{:?}` has no entity", face_entity);
            continue;
        };
        let Some(edge_b_entity) = face_3d.edge_b.get(&client) else {
            warn!("Face3d component of entity: `{:?}` has no entity", face_entity);
            continue;
        };
        let Some(edge_c_entity) = face_3d.edge_c.get(&client) else {
            warn!("Face3d component of entity: `{:?}` has no entity", face_entity);
            continue;
        };

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &face_entity,
            ShapeWaitlistInsert::Face(
                vertex_a_entity,
                vertex_b_entity,
                vertex_c_entity,
                edge_a_entity,
                edge_b_entity,
                edge_c_entity,
            ),
        );
    }
}

pub fn insert_owned_by_file_events(
    mut commands: Commands,
    client: Client,
    mut owned_by_events: EventReader<InsertComponentEvent<OwnedByFile>>,

    // for vertices
    mut camera_manager: ResMut<CameraManager>,
    mut canvas: ResMut<Canvas>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,

    owned_by_tab_q: Query<&OwnedByFile>,
    transform_q: Query<&Transform>,
) {
    // on OwnedByFile Insert Event
    for event in owned_by_events.iter() {
        let entity = event.entity;

        info!("entity: {:?} - inserted OwnedByFile", entity);

        let owned_by_file = owned_by_tab_q.get(entity).unwrap();
        let file_entity = owned_by_file.file_entity.get(&client).unwrap();

        // this leaks memory!!! some OwnedByFile components are never removed
        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &entity,
            ShapeWaitlistInsert::OwnedByFile(file_entity),
        );
    }
}

pub fn insert_file_type_events(
    mut commands: Commands,
    mut file_type_events: EventReader<InsertComponentEvent<FileType>>,

    // for vertices
    mut camera_manager: ResMut<CameraManager>,
    mut canvas: ResMut<Canvas>,
    mut vertex_manager: ResMut<VertexManager>,
    mut edge_manager: ResMut<EdgeManager>,
    mut face_manager: ResMut<FaceManager>,
    mut meshes: ResMut<Assets<CpuMesh>>,
    mut materials: ResMut<Assets<CpuMaterial>>,
    mut shape_waitlist: ResMut<ShapeWaitlist>,

    file_type_q: Query<&FileType>,
    transform_q: Query<&Transform>,
) {
    // on FileType Insert Event
    for event in file_type_events.iter() {
        let entity = event.entity;

        let file_type = file_type_q.get(entity).unwrap();
        let file_type_value = *file_type.value;

        info!(
            "entity: {:?} - inserted FileType::{:?}",
            entity, file_type_value
        );

        shape_waitlist.process_insert(
            &mut commands,
            &mut meshes,
            &mut materials,
            &mut camera_manager,
            &mut canvas,
            &mut vertex_manager,
            &mut edge_manager,
            &mut face_manager,
            &transform_q,
            &entity,
            ShapeWaitlistInsert::FileType(file_type_value),
        );
    }
}

pub fn insert_animation_events(
    mut commands: Commands,
    client: Client,
    mut animation_manager: ResMut<AnimationManager>,
    mut frame_events: EventReader<InsertComponentEvent<AnimFrame>>,
    mut rotation_events: EventReader<InsertComponentEvent<AnimRotation>>,
    frame_q: Query<&AnimFrame>,
    rotation_q: Query<&AnimRotation>,
) {
    // on AnimFrame Insert Event
    for event in frame_events.iter() {
        let frame_entity = event.entity;

        info!("entity: {:?} - inserted AnimFrame", frame_entity);

        let frame = frame_q.get(frame_entity).unwrap();
        let file_entity = frame.file_entity.get(&client).unwrap();
        let _frame_order = frame.get_order() as usize;

        animation_manager.frame_postprocess(file_entity, frame_entity);
    }

    // on AnimRotation Insert Event
    for event in rotation_events.iter() {
        let rotation_entity = event.entity;

        info!("entity: {:?} - inserted AnimRotation", rotation_entity);

        let rotation = rotation_q.get(rotation_entity).unwrap();

        let frame_entity = rotation.frame_entity.get(&client).unwrap();

        let vertex_name = (*rotation.vertex_name).clone();

        animation_manager.rotation_postprocess(
            &mut commands,
            frame_entity,
            rotation_entity,
            vertex_name,
        );
    }
}

pub fn insert_palette_events(
    client: Client,
    mut color_events: EventReader<InsertComponentEvent<PaletteColor>>,
    mut palette_manager: ResMut<PaletteManager>,
    color_q: Query<&PaletteColor>,
) {
    // on PaletteColor Insert Event
    for event in color_events.iter() {
        let color_entity = event.entity;

        info!("entity: {:?} - inserted PaletteColor", color_entity);

        let Ok(color_component) = color_q.get(color_entity) else {
            continue;
        };

        let file_entity = color_component.file_entity.get(&client).unwrap();
        let color_index = *color_component.index as usize;

        palette_manager.register_color(file_entity, color_entity, color_index);
    }
}
