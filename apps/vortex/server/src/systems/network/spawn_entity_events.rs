use bevy_ecs::{
    event::EventReader,
    system::{Commands, Query, Res, ResMut},
};
use bevy_log::info;

use naia_bevy_server::{
    events::{DespawnEntityEvent, SpawnEntityEvent},
    Server,
};

use vortex_proto::components::ChangelistEntry;

use crate::{
    files::ShapeType,
    resources::{GitManager, ShapeManager, UserManager},
};

pub fn spawn_entity_events(mut event_reader: EventReader<SpawnEntityEvent>) {
    for SpawnEntityEvent(_user_key, entity) in event_reader.iter() {
        info!("entity: `{:?}`, spawned", entity);
    }
}

pub fn despawn_entity_events(
    mut commands: Commands,
    mut server: Server,
    user_manager: Res<UserManager>,
    mut git_manager: ResMut<GitManager>,
    mut shape_manager: ResMut<ShapeManager>,
    mut event_reader: EventReader<DespawnEntityEvent>,
    mut changelist_q: Query<&mut ChangelistEntry>,
) {
    for DespawnEntityEvent(user_key, entity) in event_reader.iter() {
        let Some(user_session_data) = user_manager.user_session_data(user_key) else {
            panic!("user not found");
        };
        let project = git_manager
            .project_mut(&user_session_data.project_key().unwrap())
            .unwrap();

        let mut shape_type = None;
        if project.entity_is_file(entity) {
            shape_type = Some((true, None));
        } else if shape_manager.has_vertex(entity) {
            shape_type = Some((false, Some(ShapeType::Vertex)));
        } else if shape_manager.has_edge(entity) {
            shape_type = Some((false, Some(ShapeType::Edge)));
        } else if shape_manager.has_face(entity) {
            shape_type = Some((false, Some(ShapeType::Face)));
        }

        match shape_type {
            Some((true, None)) => {
                // file
                info!("entity: `{:?}` (which is a File), despawned", entity);

                project.on_client_delete_file(
                    &mut commands,
                    &mut server,
                    &mut changelist_q,
                    entity,
                );
            }
            Some((false, Some(ShapeType::Vertex))) => {
                // vertex
                info!("entity: `{:?}` (which is a Vertex), despawned", entity);

                let other_entities_to_despawn =
                    shape_manager.on_client_despawn_vertex(&mut commands, &mut server, entity);

                git_manager.on_client_remove_content_entity(&entity);
                for other_entity in other_entities_to_despawn {
                    git_manager.on_client_remove_content_entity(&other_entity);
                }
            }
            Some((false, Some(ShapeType::Edge))) => {
                // edge
                info!("entity: `{:?}` (which is an Edge), despawned", entity);

                shape_manager.on_client_despawn_edge(entity);

                git_manager.on_client_remove_content_entity(&entity);
            }
            Some((false, Some(ShapeType::Face))) => {
                // edge
                info!("entity: `{:?}` (which is an Face), despawned", entity);

                shape_manager.on_delete_face(entity);

                git_manager.on_client_remove_content_entity(&entity);
            }
            _ => {
                panic!(
                    "despawned entity: `{:?}` which is ({:?})",
                    entity, shape_type
                );
            }
        }
    }
}
