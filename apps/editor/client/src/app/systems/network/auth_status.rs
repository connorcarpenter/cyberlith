use bevy_ecs::{
    entity::Entity,
    event::EventReader,
    system::{Query, Res, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use logging::{info, warn};

use naia_bevy_client::{
    events::{EntityAuthDeniedEvent, EntityAuthGrantedEvent, EntityAuthResetEvent},
    Client,
};

use editor_proto::components::{
    AnimFrame, AnimRotation, BackgroundSkinColor, Edge3d, Face3d, FaceColor, FileDependency,
    FileSystemEntry, IconEdge, IconFace, IconFrame, IconVertex, NetTransform, OwnedByFile,
    PaletteColor, Vertex3d,
};

use crate::app::{
    components::OwnedByFileLocal,
    plugin::Main,
    resources::{
        action::file::FileActions, animation_manager::AnimationManager, edge_manager::EdgeManager,
        face_manager::FaceManager, shape_manager::ShapeManager, tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
};

#[derive(Resource)]
struct CachedAuthEventsState {
    event_state: SystemState<(
        EventReader<'static, 'static, EntityAuthGrantedEvent<Main>>,
        EventReader<'static, 'static, EntityAuthDeniedEvent<Main>>,
        EventReader<'static, 'static, EntityAuthResetEvent<Main>>,
    )>,
}

pub fn auth_event_startup(world: &mut World) {
    let event_state = SystemState::<(
        EventReader<EntityAuthGrantedEvent<Main>>,
        EventReader<EntityAuthDeniedEvent<Main>>,
        EventReader<EntityAuthResetEvent<Main>>,
    )>::new(world);
    world.insert_resource(CachedAuthEventsState { event_state });
}

pub fn auth_events(world: &mut World) {
    let mut auth_granted_events: Vec<Entity> = Vec::new();
    let mut auth_denied_events: Vec<Entity> = Vec::new();
    let mut auth_reset_events: Vec<Entity> = Vec::new();

    world.resource_scope(
        |world, mut events_reader_state: Mut<CachedAuthEventsState>| {
            let (mut granted_events, mut denied_events, mut reset_events) =
                events_reader_state.event_state.get_mut(world);

            for event in granted_events.read() {
                auth_granted_events.push(event.entity);
            }

            for event in denied_events.read() {
                auth_denied_events.push(event.entity);
            }

            for event in reset_events.read() {
                auth_reset_events.push(event.entity);
            }
        },
    );

    if auth_granted_events.is_empty()
        && auth_reset_events.is_empty()
        && auth_denied_events.is_empty()
    {
        return;
    }

    let mut system_state: SystemState<(
        Client<Main>,
        ResMut<FileActions>,
        ResMut<TabManager>,
        Res<VertexManager>,
        Res<EdgeManager>,
        Res<FaceManager>,
        Res<AnimationManager>,
        Query<(
            Option<&FileSystemEntry>,
            Option<&FileDependency>,
            Option<&Vertex3d>,
            Option<&Edge3d>,
            Option<&Face3d>,
            Option<&AnimFrame>,
            Option<&AnimRotation>,
            Option<&PaletteColor>,
            Option<&FaceColor>,
            Option<&BackgroundSkinColor>,
            Option<&NetTransform>,
            Option<&OwnedByFile>,
        )>,
        Query<(
            Option<&IconVertex>,
            Option<&IconEdge>,
            Option<&IconFace>,
            Option<&IconFrame>,
        )>,
        Query<&OwnedByFileLocal>,
        Query<&AnimFrame>,
    )> = SystemState::new(world);
    let (
        client,
        mut file_actions,
        mut tab_manager,
        vertex_manager,
        edge_manager,
        face_manager,
        animation_manager,
        big_q,
        big_q2,
        owned_by_q,
        anim_frame_q,
    ) = system_state.get_mut(world);

    for (entities, msg) in [
        (auth_granted_events, "granted"),
        (auth_denied_events, "denied"),
        (auth_reset_events, "reset"),
    ] {
        if entities.is_empty() {
            continue;
        }

        for entity in entities {
            process_entity_auth_status(
                &client,
                &mut file_actions,
                &mut tab_manager,
                &vertex_manager,
                &edge_manager,
                &face_manager,
                &animation_manager,
                &big_q,
                &big_q2,
                &owned_by_q,
                &anim_frame_q,
                &entity,
                msg,
            );
        }
    }
}

fn process_entity_auth_status(
    client: &Client<Main>,
    file_actions: &mut FileActions,
    tab_manager: &mut TabManager,
    vertex_manager: &VertexManager,
    edge_manager: &EdgeManager,
    face_manager: &FaceManager,
    animation_manager: &AnimationManager,
    big_q: &Query<(
        Option<&FileSystemEntry>,
        Option<&FileDependency>,
        Option<&Vertex3d>,
        Option<&Edge3d>,
        Option<&Face3d>,
        Option<&AnimFrame>,
        Option<&AnimRotation>,
        Option<&PaletteColor>,
        Option<&FaceColor>,
        Option<&BackgroundSkinColor>,
        Option<&NetTransform>,
        Option<&OwnedByFile>,
    )>,
    big_q2: &Query<(
        Option<&IconVertex>,
        Option<&IconEdge>,
        Option<&IconFace>,
        Option<&IconFrame>,
    )>,
    owned_by_q: &Query<&OwnedByFileLocal>,
    anim_frame_q: &Query<&AnimFrame>,
    entity: &Entity,
    status: &str,
) {
    let Ok((
        fs_entry_opt,
        dep_opt,
        vertex_opt,
        edge_opt,
        face_opt,
        frame_opt,
        rot_opt,
        palette_opt,
        face_color_opt,
        bckg_color_opt,
        net_transform_opt,
        owned_by_file_opt,
    )) = big_q.get(*entity)
    else {
        warn!(
            "process_entity_auth_status() for non-existent entity!: {:?}",
            entity
        );
        return;
    };

    let Ok((icon_vertex_opt, icon_edge_opt, icon_face_opt, icon_frame_opt)) = big_q2.get(*entity)
    else {
        warn!(
            "process_entity_auth_status() for non-existent entity!: {:?}",
            entity
        );
        return;
    };

    if vertex_opt.is_some() || edge_opt.is_some() || face_opt.is_some() {
        info!(
            "auth processing for shape entity `{:?}`: `{:?}`",
            entity, status
        );
        if let Ok(owning_file_entity) = owned_by_q.get(*entity) {
            if let Some(tab_state) = tab_manager.tab_state_mut(&owning_file_entity.file_entity) {
                let shape_2d_entity = ShapeManager::shape_entity_3d_to_2d(
                    vertex_manager,
                    edge_manager,
                    face_manager,
                    entity,
                )
                .unwrap();
                tab_state
                    .action_stack
                    .entity_update_auth_status(&shape_2d_entity);
            } else {
                warn!(
                    "no tab state found for file entity: {:?}",
                    owning_file_entity.file_entity
                );
            }
        } else {
            warn!("no owning file entity found for shape entity: {:?}", entity);
        }
    } else if fs_entry_opt.is_some() {
        // entity is file
        info!(
            "auth processing for file entity `{:?}`: `{:?}`",
            entity, status
        );
        file_actions.entity_update_auth_status(entity);
    } else if dep_opt.is_some() {
        // entity is dependency
        info!(
            "auth processing for dependency entity `{:?}`: `{:?}`",
            entity, status
        );
        file_actions.entity_update_auth_status(entity);
    } else if let Some(frame_component) = frame_opt {
        info!(
            "auth processing for frame entity `{:?}`: `{:?}`",
            entity, status
        );
        let owning_file_entity = frame_component.file_entity.get(client).unwrap();
        if let Some(tab_state) = tab_manager.tab_state_mut(&owning_file_entity) {
            tab_state.action_stack.entity_update_auth_status(&entity);
        } else {
            warn!(
                "no tab state found for file entity: {:?}",
                owning_file_entity
            );
        }
    } else if rot_opt.is_some() {
        info!(
            "auth processing for rotation entity `{:?}`: `{:?}`",
            entity, status
        );
        let frame_entity = animation_manager
            .get_rotations_frame_entity(entity)
            .unwrap();
        let Ok(frame_component) = anim_frame_q.get(frame_entity) else {
            panic!(
                "component for rotation entity `{:?}` not found",
                frame_entity
            );
        };
        let owning_file_entity = frame_component.file_entity.get(client).unwrap();
        if let Some(tab_state) = tab_manager.tab_state_mut(&owning_file_entity) {
            tab_state.action_stack.entity_update_auth_status(&entity);
        } else {
            warn!(
                "no tab state found for file entity: {:?}",
                owning_file_entity
            );
        }
    } else if let Some(color_component) = palette_opt {
        info!(
            "auth processing for color entity `{:?}`: `{:?}`",
            entity, status
        );
        let owning_file_entity = color_component.owning_file_entity.get(client).unwrap();
        if let Some(tab_state) = tab_manager.tab_state_mut(&owning_file_entity) {
            tab_state.action_stack.entity_update_auth_status(&entity);
        } else {
            warn!(
                "no tab state found for file entity: {:?}",
                owning_file_entity
            );
        }
    } else if let Some(color_component) = face_color_opt {
        info!(
            "auth processing for face color entity `{:?}`: `{:?}`",
            entity, status
        );
        let owning_file_entity = color_component.owning_file_entity.get(client).unwrap();
        if let Some(tab_state) = tab_manager.tab_state_mut(&owning_file_entity) {
            tab_state.action_stack.entity_update_auth_status(&entity);
        } else {
            warn!(
                "no tab state found for file entity: {:?}",
                owning_file_entity
            );
        }
    } else if bckg_color_opt.is_some() {
        info!(
            "auth processing for background skin color entity `{:?}`: `{:?}`",
            entity, status
        );
        // no need to set auth status on action stack because auth for background color is automatically given (and reset upon update)
    } else if let Some(_owned_by_component) = net_transform_opt {
        info!(
            "auth processing for net transform entity `{:?}`: `{:?}`",
            entity, status
        );
        let owned_by_component = owned_by_file_opt.unwrap();
        let owning_file_entity = owned_by_component.file_entity.get(client).unwrap();
        if let Some(tab_state) = tab_manager.tab_state_mut(&owning_file_entity) {
            tab_state.action_stack.entity_update_auth_status(&entity);
        } else {
            warn!(
                "no tab state found for file entity: {:?}",
                owning_file_entity
            );
        }
    } else if icon_vertex_opt.is_some() || icon_edge_opt.is_some() || icon_face_opt.is_some() {
        info!(
            "auth processing for shape entity `{:?}`: `{:?}`",
            entity, status
        );
        if let Ok(owning_file_entity) = owned_by_q.get(*entity) {
            if let Some(tab_state) = tab_manager.tab_state_mut(&owning_file_entity.file_entity) {
                tab_state.action_stack.entity_update_auth_status(&entity);
            } else {
                warn!(
                    "no tab state found for file entity: {:?}",
                    owning_file_entity.file_entity
                );
            }
        } else {
            warn!(
                "no owning file entity found for icon shape entity: {:?}",
                entity
            );
        }
    } else if let Some(frame_component) = icon_frame_opt {
        info!(
            "auth processing for icon frame entity `{:?}`: `{:?}`",
            entity, status
        );
        let owning_file_entity = frame_component.file_entity.get(client).unwrap();
        if let Some(tab_state) = tab_manager.tab_state_mut(&owning_file_entity) {
            tab_state.action_stack.entity_update_auth_status(&entity);
        } else {
            warn!(
                "no tab state found for file entity: {:?}",
                owning_file_entity
            );
        }
    } else {
        warn!("unhandled auth status: entity `{:?}`: {:?}", entity, status);
    }
}
