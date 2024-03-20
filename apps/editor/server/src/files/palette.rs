use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{CommandsExt, ReplicationConfig, Server};

use asset_id::AssetId;
use asset_io::json::PaletteJson;

use editor_proto::components::PaletteColor;

use crate::{
    files::FileWriter,
    resources::{ContentEntityData, PaletteManager, Project},
};

// Writer
pub struct PaletteWriter;

impl PaletteWriter {
    fn world_to_data(&self, world: &mut World) -> PaletteJson {
        let mut system_state: SystemState<Query<&PaletteColor>> = SystemState::new(world);
        let color_q = system_state.get_mut(world);

        let mut output = PaletteJson::new();

        for color in color_q.iter() {
            let index = *color.index as usize;
            let r = *color.r;
            let g = *color.g;
            let b = *color.b;

            output.insert_color(index, r, g, b);
        }

        output
    }
}

impl FileWriter for PaletteWriter {
    fn write(
        &self,
        world: &mut World,
        _project: &Project,
        _content_entities: &HashMap<Entity, ContentEntityData>,
        asset_id: &AssetId,
    ) -> Box<[u8]> {
        let data = self.world_to_data(world);
        data.write(asset_id)
    }

    fn write_new_default(&self, asset_id: &AssetId) -> Box<[u8]> {
        let mut data = PaletteJson::new();
        data.add_color(255, 255, 255); // white
        data.add_color(0, 0, 0); // black
        data.add_color(255, 0, 0); // red
        data.add_color(0, 255, 0); // green
        data.add_color(0, 0, 255); // blue
        data.add_color(255, 255, 0); // yellow
        data.add_color(0, 255, 255); // cyan
        data.add_color(255, 0, 255); // magenta
        data.write(asset_id)
    }
}

// Reader
pub struct PaletteReader;

impl PaletteReader {
    fn data_to_world(
        world: &mut World,
        file_entity: &Entity,
        data: &PaletteJson,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut output = HashMap::new();
        let mut index = 0;

        let mut system_state: SystemState<(Commands, Server, ResMut<PaletteManager>)> =
            SystemState::new(world);
        let (mut commands, mut server, mut palette_manager) = system_state.get_mut(world);

        for color in data.get_colors() {
            let (r, g, b) = color.deconstruct();
            let mut color_component = PaletteColor::new(index, r, g, b);
            color_component
                .owning_file_entity
                .set(&mut server, file_entity);
            let entity_id = commands
                .spawn_empty()
                .enable_replication(&mut server)
                .configure_replication(ReplicationConfig::Delegated)
                .insert(color_component)
                .id();
            info!(
                "palette color entity: `{:?}`, rgb:({}, {}, {})",
                entity_id, r, g, b
            );
            output.insert(entity_id, ContentEntityData::new_palette_color());

            palette_manager.on_create_color(&file_entity, &entity_id, index as usize, None);

            index += 1;
        }

        system_state.apply(world);

        output
    }

    pub fn read(
        &self,
        world: &mut World,
        file_entity: &Entity,
        bytes: &Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData> {
        let Ok((meta, data)) = PaletteJson::read(bytes) else {
            panic!("Error reading .palette file");
        };

        if meta.schema_version() != PaletteJson::CURRENT_SCHEMA_VERSION {
            panic!("Invalid schema version");
        }

        let result = Self::data_to_world(world, file_entity, &data);

        result
    }
}
