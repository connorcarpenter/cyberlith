use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{SystemState, Query, Res},
};
use bevy_log::info;

use naia_bevy_server::{BitReader, CommandsExt, FileBitWriter, ReplicationConfig, Serde, SerdeErr, Server};

use vortex_proto::{components::{BackgroundSkinColor, FileExtension, FaceColor, PaletteColor}, resources::FileKey};

use crate::{
    files::{add_file_dependency, FileWriter},
    resources::{ShapeManager, ContentEntityData, Project},
};

// Actions
#[derive(Clone)]
enum SkinAction {
    // path, file_name
    PaletteFile(String, String),
    // path, file_name
    MeshFile(String, String),
    // palette color index
    BackgroundColor(u8),
    // mesh face index, palette color index
    SkinColor(u16, u8),
}

#[derive(Serde, Clone, PartialEq)]
enum SkinActionType {
    PaletteFile,
    MeshFile,
    BackgroundColor,
    SkinColor,
    None,
}

// Writer
pub struct SkinWriter;

impl SkinWriter {
    fn world_to_actions(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Vec<SkinAction> {
        let working_file_entries = project.working_file_entries();

        let mut palette_dependency_key_opt = None;
        let mut mesh_dependency_key_opt = None;
        let mut face_color_entities = Vec::new();

        for (content_entity, content_data) in content_entities {
            match content_data {
                ContentEntityData::Dependency(dependency_key) => {
                    let dependency_value = working_file_entries.get(dependency_key).unwrap();
                    let dependency_file_ext = dependency_value.extension().unwrap();
                    match dependency_file_ext {
                        FileExtension::Palette => {
                            palette_dependency_key_opt = Some(dependency_key);
                        }
                        FileExtension::Mesh => {
                            mesh_dependency_key_opt = Some(dependency_key);
                        }
                        _ => {
                            panic!("skin file should depend on a single .mesh file & a single .palette");
                        }
                    }
                }
                ContentEntityData::FaceColor(_) => {
                    face_color_entities.push(*content_entity);
                }
                _ => {
                    panic!("skin should not have this content entity type");
                }
            }
        }

        let mut actions = Vec::new();

        // Write Palette Dependency
        if let Some(dependency_key) = palette_dependency_key_opt {
            info!("writing palette dependency: {}", dependency_key.full_path());
            actions.push(SkinAction::PaletteFile(
                dependency_key.path().to_string(),
                dependency_key.name().to_string(),
            ));
        }

        // Write Mesh Dependency
        if let Some(dependency_key) = mesh_dependency_key_opt {
            info!("writing mesh dependency: {}", dependency_key.full_path());
            actions.push(SkinAction::MeshFile(
                dependency_key.path().to_string(),
                dependency_key.name().to_string(),
            ));
        }

        for face_color_entity in face_color_entities {
            let mut system_state: SystemState<(
                Server,
                Res<ShapeManager>,
                Query<&PaletteColor>,
                Query<&FaceColor>
            )> = SystemState::new(world);
            let (
                server,
                shape_manager,
                palette_color_q,
                face_color_q
            ) = system_state.get_mut(world);

            let face_color = face_color_q.get(face_color_entity).unwrap();

            let face_3d_entity = face_color.face_3d_entity.get(&server).unwrap();
            let face_index = shape_manager.get_face_index(&face_3d_entity).unwrap() as u16;

            let palette_entity = face_color.palette_color_entity.get(&server).unwrap();
            let palette_color = palette_color_q.get(palette_entity).unwrap();
            let palette_color_index = *palette_color.index;

            actions.push(SkinAction::SkinColor(face_index, palette_color_index));
        }

        actions
    }

    fn write_from_actions(&self, actions: Vec<SkinAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                SkinAction::PaletteFile(path, file_name) => {
                    SkinActionType::PaletteFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
                SkinAction::MeshFile(path, file_name) => {
                    SkinActionType::MeshFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
                SkinAction::BackgroundColor(palette_color_index) => {
                    SkinActionType::BackgroundColor.ser(&mut bit_writer);

                    // TODO: could optimize these a bit more .. unlikely to use all these bits
                    palette_color_index.ser(&mut bit_writer);
                }
                SkinAction::SkinColor(face_index, palette_color_index) => {
                    SkinActionType::SkinColor.ser(&mut bit_writer);

                    // TODO: could optimize these a bit more .. unlikely to use all these bits
                    face_index.ser(&mut bit_writer);
                    palette_color_index.ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        SkinActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

impl FileWriter for SkinWriter {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let actions = self.world_to_actions(world, project, content_entities);
        self.write_from_actions(actions)
    }

    fn write_new_default(&self) -> Box<[u8]> {
        let mut actions = Vec::new();

        actions.push(SkinAction::BackgroundColor(0));

        self.write_from_actions(actions)
    }
}

// Reader
pub struct SkinReader;

impl SkinReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<SkinAction>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = SkinActionType::de(bit_reader)?;

            match action_type {
                SkinActionType::PaletteFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    actions.push(SkinAction::PaletteFile(path, file_name));
                }
                SkinActionType::MeshFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    actions.push(SkinAction::MeshFile(path, file_name));
                }
                SkinActionType::BackgroundColor => {
                    let palette_color_index = u8::de(bit_reader)?;
                    actions.push(SkinAction::BackgroundColor(palette_color_index));
                }
                SkinActionType::SkinColor => {
                    let face_index = u16::de(bit_reader)?;
                    let palette_color_index = u8::de(bit_reader)?;
                    actions.push(SkinAction::SkinColor(face_index, palette_color_index));
                }
                SkinActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }

    fn actions_to_world(
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        actions: Vec<SkinAction>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut output = HashMap::new();

        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (mut commands, mut server) = system_state.get_mut(world);

        for action in actions {
            match action {
                SkinAction::PaletteFile(palette_path, palette_file_name) => {
                    let (new_entity, new_file_key) = add_file_dependency(
                        project,
                        file_key,
                        file_entity,
                        &mut commands,
                        &mut server,
                        FileExtension::Palette,
                        &palette_path,
                        &palette_file_name,
                    );
                    output.insert(new_entity, ContentEntityData::new_dependency(new_file_key));
                }
                SkinAction::MeshFile(mesh_path, mesh_file_name) => {
                    let (new_entity, new_file_key) = add_file_dependency(
                        project,
                        file_key,
                        file_entity,
                        &mut commands,
                        &mut server,
                        FileExtension::Mesh,
                        &mesh_path,
                        &mesh_file_name,
                    );
                    output.insert(new_entity, ContentEntityData::new_dependency(new_file_key));
                }
                SkinAction::BackgroundColor(palette_index) => {
                    let mut background_color_component = BackgroundSkinColor::new();
                    background_color_component.skin_file_entity.set(&server, file_entity);

                    let entity_id = commands
                        .spawn_empty()
                        .enable_replication(&mut server)
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(background_color_component)
                        .id();
                    info!("spawning background skin color entity {:?}", entity_id);
                    output.insert(entity_id, ContentEntityData::new_background_skin_color(Some(palette_index)));
                }
                SkinAction::SkinColor(face_index, palette_index) => {

                    let mut face_color_component = FaceColor::new();
                    face_color_component.skin_file_entity.set(&server, file_entity);

                    let entity_id = commands
                        .spawn_empty()
                        .enable_replication(&mut server)
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(face_color_component)
                        .id();
                    info!("spawning face color entity {:?}", entity_id);
                    output.insert(entity_id, ContentEntityData::new_skin_color(Some((face_index, palette_index))));
                }
            }
        }

        system_state.apply(world);

        output
    }

    pub fn read(
        &self,
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        bytes: &Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut bit_reader = BitReader::new(bytes);

        let Ok(actions) = Self::read_to_actions(&mut bit_reader) else {
            panic!("Error reading .skin file");
        };

        let result = Self::actions_to_world(world, project, file_key, file_entity, actions);

        result
    }
}
