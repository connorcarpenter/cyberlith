use bevy_ecs::{
    event::EventReader,
    system::{Commands, Query, Res, ResMut},
};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, EntityAuthStatus, events::UpdateComponentEvents, Server};

use vortex_proto::components::{AnimFrame, AnimRotation, BackgroundSkinColor, EdgeAngle, FaceColor, FileSystemChild, FileSystemEntry, PaletteColor, ShapeName, Vertex3d};

use crate::resources::{GitManager, UserManager};

pub fn update_component_events(
    mut commands: Commands,
    mut server: Server,
    mut git_manager: ResMut<GitManager>,
    user_manager: Res<UserManager>,
    mut event_reader: EventReader<UpdateComponentEvents>,
    shape_name_q: Query<&ShapeName>,
) {
    for events in event_reader.iter() {
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
            let Some((project_key, file_key)) = git_manager.content_entity_keys(&entity) else {
                panic!("no content entity keys!");
            };
            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
        }
        // on EdgeAngle Update Event
        for (_, entity) in events.read::<EdgeAngle>() {
            let Some((project_key, file_key)) = git_manager.content_entity_keys(&entity) else {
                panic!("no content entity keys!");
            };
            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
        }
        // on ShapeName Update Event
        for (_, entity) in events.read::<ShapeName>() {
            let shape_name = shape_name_q.get(entity).unwrap();
            info!(
                "entity: {:?} updated ShapeName to: {:?}",
                entity, *shape_name.value
            );

            let Some((project_key, file_key)) = git_manager.content_entity_keys(&entity) else {
                panic!("no content entity keys!");
            };
            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
        }
        // on AnimFrame Update Event
        for (_, entity) in events.read::<AnimFrame>() {
            let Some((project_key, file_key)) = git_manager.content_entity_keys(&entity) else {
                panic!("no content entity keys!");
            };
            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
        }
        // on AnimRotation Update Event
        for (_, entity) in events.read::<AnimRotation>() {
            let Some((project_key, file_key)) = git_manager.content_entity_keys(&entity) else {
                panic!("no content entity keys!");
            };
            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
        }
        // on PaletteColor Update Event
        for (_, entity) in events.read::<PaletteColor>() {
            let Some((project_key, file_key)) = git_manager.content_entity_keys(&entity) else {
                panic!("no content entity keys!");
            };
            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
        }
        // on BackgroundSkinColor Update Event
        for (_, entity) in events.read::<BackgroundSkinColor>() {
            let Some((project_key, file_key)) = git_manager.content_entity_keys(&entity) else {
                panic!("no content entity keys!");
            };

            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);

            let auth = commands.entity(entity).authority(&server).unwrap();
            if auth != EntityAuthStatus::Available {
                info!("reset bck color component auth status");
                commands.entity(entity).take_authority(&mut server);
            }
        }
        // on FaceColor Update Event
        for (_, entity) in events.read::<FaceColor>() {
            let Some((project_key, file_key)) = git_manager.content_entity_keys(&entity) else {
                panic!("no content entity keys!");
            };
            git_manager.on_client_modify_file(&mut commands, &mut server, &project_key, &file_key);
        }
    }
}
