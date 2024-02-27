use bevy_ecs::{
    event::EventReader,
    system::{Commands, Query, Res, ResMut},
};
use bevy_log::{info, warn};

use naia_bevy_server::{events::UpdateComponentEvents, CommandsExt, EntityAuthStatus, Server};

use editor_proto::components::{
    AnimFrame, AnimRotation, BackgroundSkinColor, EdgeAngle, FaceColor, FileSystemChild,
    FileSystemEntry, IconFace, IconFrame, IconVertex, NetTransform, PaletteColor, ShapeName,
    Vertex3d,
};

use crate::resources::{GitManager, IconManager, UserManager};

pub fn update_component_events(
    mut commands: Commands,
    mut server: Server,
    mut git_manager: ResMut<GitManager>,
    user_manager: Res<UserManager>,
    mut icon_manager: ResMut<IconManager>,
    mut event_reader: EventReader<UpdateComponentEvents>,
    shape_name_q: Query<&ShapeName>,
    mut icon_frame_q: Query<&mut IconFrame>,
) {
    let mut modified_content_entities = Vec::new();
    for events in event_reader.read() {
        // on FileSystemEntry Update Event
        for (user_key, entity) in events.read::<FileSystemEntry>() {
            let username = user_manager
                .user_session_data(&user_key)
                .unwrap()
                .username();
            let project_key = git_manager.project_key_from_name(username).unwrap();
            let project = git_manager.project(&project_key).unwrap();
            let file_key = project.get_file_key_from_entity(&entity).unwrap();
            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
        }
        // on FileSystemChild Update Event
        for (_user_key, _entity) in events.read::<FileSystemChild>() {
            // TODO!
        }
        // on Vertex3D Update Event
        for (_, entity) in events.read::<Vertex3d>() {
            modified_content_entities.push(entity);
        }
        // on IconVertex Update Event
        for (_, entity) in events.read::<IconVertex>() {
            modified_content_entities.push(entity);
        }
        // on IconFace Update Event
        for (_, entity) in events.read::<IconFace>() {
            modified_content_entities.push(entity);
        }
        // on IconFrame Update Event
        for (_, entity) in events.read::<IconFrame>() {

            let Ok(frame) = icon_frame_q.get(entity) else {
                panic!("icon frame not found");
            };
            let updated_file_entity = frame.file_entity.get(&server).unwrap();
            let updated_frame_order = frame.get_order();
            icon_manager.update_frame_order(&updated_file_entity, &entity, updated_frame_order, &mut icon_frame_q);

            info!("---");

            let mut frame_list = Vec::new();

            for frame in icon_frame_q.iter() {
                let frames_file_entity = frame.file_entity.get(&server).unwrap();
                if frames_file_entity != updated_file_entity {
                    continue;
                }
                let frame_index = frame.get_order() as usize;
                let frame_entity = icon_manager.get_frame_at_index(&frames_file_entity, frame_index).unwrap();
                // resize if necessary
                if frame_index >= frame_list.len() {
                    frame_list.resize(frame_index + 1, None);
                }
                if frame_list[frame_index].is_some() {
                    warn!("Duplicate frame order! {:?}", frame_index);
                }
                frame_list[frame_index] = Some(frame_entity);
            }

            for (index, frame_entity_opt) in frame_list.iter().enumerate() {
                info!("frame order: {:?} -> {:?}", index, frame_entity_opt);
            }

            info!("---");

            modified_content_entities.push(entity);
        }
        // on EdgeAngle Update Event
        for (_, entity) in events.read::<EdgeAngle>() {
            modified_content_entities.push(entity);
        }
        // on ShapeName Update Event
        for (_, entity) in events.read::<ShapeName>() {
            let shape_name = shape_name_q.get(entity).unwrap();
            info!(
                "entity: {:?} updated ShapeName to: {:?}",
                entity, *shape_name.value
            );

            modified_content_entities.push(entity);
        }
        // on AnimFrame Update Event
        for (_, entity) in events.read::<AnimFrame>() {
            modified_content_entities.push(entity);
        }
        // on AnimRotation Update Event
        for (_, entity) in events.read::<AnimRotation>() {
            modified_content_entities.push(entity);
        }
        // on PaletteColor Update Event
        for (_, entity) in events.read::<PaletteColor>() {
            modified_content_entities.push(entity);
        }
        // on BackgroundSkinColor Update Event
        for (_, entity) in events.read::<BackgroundSkinColor>() {
            modified_content_entities.push(entity);

            let auth = commands.entity(entity).authority(&server).unwrap();
            if auth != EntityAuthStatus::Available {
                info!("reset bck color component auth status");
                commands.entity(entity).take_authority(&mut server);
            }
        }
        // on FaceColor Update Event
        for (_, entity) in events.read::<FaceColor>() {
            modified_content_entities.push(entity);
        }
        // on NetTransform Update Event
        for (_, entity) in events.read::<NetTransform>() {
            modified_content_entities.push(entity);
        }
    }

    for modified_content_entity in modified_content_entities {
        let Some((project_key, file_key)) =
            git_manager.content_entity_keys(&modified_content_entity)
        else {
            panic!("no content entity keys!");
        };
        git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
    }
}
