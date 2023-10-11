use bevy_ecs::{
    event::EventReader,
    system::{Commands, Query, Res, ResMut},
};
use bevy_log::info;

use naia_bevy_server::{
    events::{DespawnEntityEvent, SpawnEntityEvent},
    Server,
};

use vortex_proto::components::{AnimFrame, ChangelistEntry, PaletteColor};

use crate::resources::{AnimationManager, GitManager, PaletteManager, ShapeManager, SkinManager, UserManager};

pub fn spawn_entity_events(mut event_reader: EventReader<SpawnEntityEvent>) {
    for SpawnEntityEvent(_user_key, entity) in event_reader.iter() {
        info!("entity: `{:?}`, spawned", entity);
    }
}

#[derive(Debug)]
enum DespawnType {
    File,
    Vertex,
    Edge,
    Face,
    Rotation,
    Frame,
    PaletteColor,
    FaceColor,
}

pub fn despawn_entity_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut shape_manager: ResMut<ShapeManager>,
    mut animation_manager: ResMut<AnimationManager>,
    mut palette_manager: ResMut<PaletteManager>,
    mut skin_manager: ResMut<SkinManager>,
    mut event_reader: EventReader<DespawnEntityEvent>,
    mut changelist_q: Query<&mut ChangelistEntry>,
    mut frame_q: Query<&mut AnimFrame>,
    mut color_q: Query<&mut PaletteColor>,
) {
    for DespawnEntityEvent(user_key, entity) in event_reader.iter() {
        let Some(user_session_data) = user_manager.user_session_data(user_key) else {
            panic!("user not found");
        };
        let project = git_manager
            .project_mut(&user_session_data.project_key().unwrap())
            .unwrap();

        let mut despawn_type = None;
        if project.entity_is_file(entity) {
            despawn_type = Some(DespawnType::File);
        } else if shape_manager.has_vertex(entity) {
            despawn_type = Some(DespawnType::Vertex);
        } else if shape_manager.has_edge(entity) {
            despawn_type = Some(DespawnType::Edge);
        } else if shape_manager.has_face(entity) {
            despawn_type = Some(DespawnType::Face);
        } else if animation_manager.has_rotation(entity) {
            despawn_type = Some(DespawnType::Rotation);
        } else if animation_manager.has_frame(entity) {
            despawn_type = Some(DespawnType::Frame);
        } else if palette_manager.has_color(entity) {
            despawn_type = Some(DespawnType::PaletteColor);
        } else if skin_manager.has_face_color(entity) {
            despawn_type = Some(DespawnType::FaceColor);
        }

        match despawn_type {
            Some(DespawnType::File) => {
                // file
                info!("entity: `{:?}` (which is a File), despawned", entity);

                project.on_client_delete_file(
                    &mut commands,
                    &mut server,
                    &mut changelist_q,
                    entity,
                );
            }
            Some(DespawnType::Vertex) => {
                // vertex
                info!("entity: `{:?}` (which is a Vertex), despawned", entity);

                git_manager.queue_client_modify_file(entity);

                let other_entities_to_despawn =
                    shape_manager.on_client_despawn_vertex(&mut commands, &mut server, entity);

                git_manager.on_remove_content_entity(&mut server, &entity);
                for other_entity in other_entities_to_despawn {
                    git_manager.on_remove_content_entity(&mut server, &other_entity);
                }
            }
            Some(DespawnType::Edge) => {
                // edge
                info!("entity: `{:?}` (which is an Edge), despawned", entity);

                git_manager.queue_client_modify_file(entity);

                shape_manager.on_client_despawn_edge(entity);

                git_manager.on_remove_content_entity(&mut server, &entity);
            }
            Some(DespawnType::Face) => {
                // face
                info!("entity: `{:?}` (which is an Face), despawned", entity);

                git_manager.queue_client_modify_file(entity);

                shape_manager.on_client_despawn_face(entity);

                git_manager.on_remove_content_entity(&mut server, &entity);
            }
            Some(DespawnType::Frame) => {
                // frame
                info!("entity: `{:?}` (which is an Frame), despawned", entity);

                git_manager.queue_client_modify_file(entity);

                animation_manager.on_despawn_frame(
                    &mut commands,
                    &mut server,
                    entity,
                    Some(&mut frame_q),
                );

                git_manager.on_remove_content_entity(&mut server, &entity);
            }
            Some(DespawnType::Rotation) => {
                // rotation
                info!("entity: `{:?}` (which is an Rotation), despawned", entity);

                git_manager.queue_client_modify_file(entity);

                animation_manager.on_despawn_rotation(entity);

                git_manager.on_remove_content_entity(&mut server, &entity);
            }
            Some(DespawnType::PaletteColor) => {
                // color
                info!("entity: `{:?}` (which is a Palette Color), despawned", entity);

                git_manager.queue_client_modify_file(entity);

                palette_manager.on_despawn_color(entity, Some(&mut color_q));

                git_manager.on_remove_content_entity(&mut server, &entity);
            }
            Some(DespawnType::FaceColor) => {
                // color
                info!("entity: `{:?}` (which is a Face Color), despawned", entity);

                git_manager.queue_client_modify_file(entity);

                skin_manager.on_despawn_face_color(entity);

                git_manager.on_remove_content_entity(&mut server, &entity);
            }
            _ => {
                panic!(
                    "despawned entity: `{:?}` which is ({:?})",
                    entity, despawn_type
                );
            }
        }
    }
}
