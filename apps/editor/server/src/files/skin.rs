use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, Res, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, ReplicationConfig, Server};

use asset_id::AssetId;
use asset_io::json::SkinJson;

use editor_proto::{
    components::{BackgroundSkinColor, FaceColor, FileExtension, PaletteColor},
    resources::FileKey,
};

use crate::{
    files::{add_file_dependency, FileWriter},
    resources::{ContentEntityData, Project, ShapeManager},
};

// Writer
pub struct SkinWriter;

impl SkinWriter {
    fn world_to_data(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> SkinJson {
        let working_file_entries = project.working_file_entries();

        let mut palette_dependency_key_opt = None;
        let mut mesh_dependency_key_opt = None;
        let mut bckg_color_entity = None;
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
                ContentEntityData::BackgroundColor(_) => {
                    bckg_color_entity = Some(*content_entity);
                }
                ContentEntityData::FaceColor(_) => {
                    face_color_entities.push(*content_entity);
                }
                _ => {
                    panic!("skin should not have this content entity type");
                }
            }
        }

        let mut output = SkinJson::new();

        // Write Palette Dependency
        if let Some(dependency_key) = palette_dependency_key_opt {
            info!("writing palette dependency: {}", dependency_key.full_path());
            let asset_id = project.asset_id(dependency_key).unwrap();
            output.set_palette_asset_id(&asset_id);
        }

        // Write Mesh Dependency
        if let Some(dependency_key) = mesh_dependency_key_opt {
            info!("writing mesh dependency: {}", dependency_key.full_path());
            let asset_id = project.asset_id(dependency_key).unwrap();
            output.set_mesh_asset_id(&asset_id);
        }

        // Write Background Color
        if let Some(bckg_entity) = bckg_color_entity {
            info!("writing background color");
            let mut system_state: SystemState<(
                Server,
                Query<&PaletteColor>,
                Query<&BackgroundSkinColor>,
            )> = SystemState::new(world);
            let (server, palette_color_q, bckg_color_q) = system_state.get_mut(world);

            let bckg_color = bckg_color_q.get(bckg_entity).unwrap();

            let palette_entity = bckg_color.palette_color_entity.get(&server).unwrap();
            let palette_color = palette_color_q.get(palette_entity).unwrap();
            let palette_color_index = *palette_color.index;

            output.set_background_color_id(palette_color_index);
        }

        for face_color_entity in face_color_entities {
            let mut system_state: SystemState<(
                Server,
                Res<ShapeManager>,
                Query<&PaletteColor>,
                Query<&FaceColor>,
            )> = SystemState::new(world);
            let (server, shape_manager, palette_color_q, face_color_q) =
                system_state.get_mut(world);

            let face_color = face_color_q.get(face_color_entity).unwrap();

            let face_3d_entity = face_color.face_entity.get(&server).unwrap();
            let face_index = shape_manager.get_face_index(&face_3d_entity).unwrap() as u16;

            let palette_entity = face_color.palette_color_entity.get(&server).unwrap();
            let palette_color = palette_color_q.get(palette_entity).unwrap();
            let palette_color_index = *palette_color.index;

            output.add_face_color(face_index, palette_color_index);
        }

        output
    }
}

impl FileWriter for SkinWriter {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
        asset_id: &AssetId,
    ) -> Box<[u8]> {
        let data = self.world_to_data(world, project, content_entities);
        data.write(asset_id)
    }

    fn write_new_default(&self, asset_id: &AssetId) -> Box<[u8]> {
        let data = SkinJson::new();
        data.write(asset_id)
    }
}

// Reader
pub struct SkinReader;

impl SkinReader {
    fn data_to_world(
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        data: &SkinJson,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut output = HashMap::new();

        let mut system_state: SystemState<(Commands, Server)> = SystemState::new(world);
        let (mut commands, mut server) = system_state.get_mut(world);

        // Palette Dependency
        let palette_asset_id = data.get_palette_asset_id();
        let dependency_file_key = project.file_key_from_asset_id(&palette_asset_id).unwrap();
        let (new_entity, _) = add_file_dependency(
            project,
            file_key,
            file_entity,
            &mut commands,
            &mut server,
            FileExtension::Palette,
            &dependency_file_key,
        );
        output.insert(
            new_entity,
            ContentEntityData::new_dependency(dependency_file_key),
        );

        // Mesh Dependency
        let mesh_asset_id = data.get_mesh_asset_id();
        let dependency_file_key = project.file_key_from_asset_id(&mesh_asset_id).unwrap();
        let (new_entity, _) = add_file_dependency(
            project,
            file_key,
            file_entity,
            &mut commands,
            &mut server,
            FileExtension::Mesh,
            &dependency_file_key,
        );
        output.insert(
            new_entity,
            ContentEntityData::new_dependency(dependency_file_key),
        );

        // Background Color
        let bckg_color = data.get_background_color_id();
        let mut background_color_component = BackgroundSkinColor::new();
        background_color_component
            .owning_file_entity
            .set(&server, file_entity);

        let entity_id = commands
            .spawn_empty()
            .enable_replication(&mut server)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(background_color_component)
            .id();
        info!("spawning background skin color entity {:?}", entity_id);
        output.insert(
            entity_id,
            ContentEntityData::new_background_color(Some(bckg_color)),
        );

        // Skin Colors
        for color in data.get_face_colors() {
            let face_index = color.face_id();
            let palette_index = color.color_id();

            let mut face_color_component = FaceColor::new();
            face_color_component
                .owning_file_entity
                .set(&server, file_entity);

            let entity_id = commands
                .spawn_empty()
                .enable_replication(&mut server)
                .configure_replication(ReplicationConfig::Delegated)
                .insert(face_color_component)
                .id();
            info!("spawning face color entity {:?}", entity_id);
            output.insert(
                entity_id,
                ContentEntityData::new_skin_color(Some((face_index, palette_index))),
            );
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
        let Ok((meta, data)) = SkinJson::read(bytes) else {
            panic!("Error reading .skin file");
        };

        if meta.schema_version() != SkinJson::CURRENT_SCHEMA_VERSION {
            panic!("Invalid schema version");
        }

        let result = Self::data_to_world(world, project, file_key, file_entity, &data);

        result
    }
}
