use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    prelude::Query,
    system::Resource,
};
use bevy_log::info;

use vortex_proto::components::PaletteColor;

pub struct FileColorData {
    // color entity -> color data
    colors: HashSet<Entity>,
    color_list: Vec<Option<Entity>>,
}

impl FileColorData {
    fn new() -> Self {
        Self {
            colors: HashSet::new(),
            color_list: Vec::new(),
        }
    }

    fn add_color(
        &mut self,
        color_entity: Entity,
        color_order: usize,
        mut color_q_opt: Option<&mut Query<&mut PaletteColor>>,
    ) {
        info!("--- add color ---");
        for i in 0..self.color_list.len() {
            info!("index: {}, entity: {:?}", i, self.color_list[i]);
        }
        info!("- op -");

        self.colors.insert(color_entity);

        // add to color_list
        if color_order >= self.color_list.len() {
            self.color_list.resize(color_order + 1, None);
            // set color entity
            self.color_list[color_order] = Some(color_entity);
        } else {
            info!(
                "add_color: index: {:?}, entity: `{:?}`",
                color_order, color_entity
            );
            self.color_list.insert(color_order, Some(color_entity));

            // move all elements after color_order up one
            for i in color_order + 1..self.color_list.len() {
                // update color_order in AnimColor using color_q_opt
                if let Some(color_q) = color_q_opt.as_mut() {
                    let Ok(mut color) = color_q.get_mut(self.color_list[i].unwrap()) else {
                        panic!("color not found");
                    };
                    *color.index = i as u8;
                }
            }
        }

        info!("--- result ---");
        for i in 0..self.color_list.len() {
            info!("index: {}, entity: {:?}", i, self.color_list[i]);
        }
    }

    fn remove_color(
        &mut self,
        color_entity: &Entity,
        color_q_opt: Option<&mut Query<&mut PaletteColor>>,
    ) -> bool {
        let result = self.colors.remove(color_entity);
        if !result {
            panic!("color entity not found");
        }

        let color_order = {
            let mut color_order_opt = None;
            for (color_index, color_item) in self.color_list.iter().enumerate() {
                if let Some(color_item) = color_item {
                    if color_item == color_entity {
                        color_order_opt = Some(color_index);
                        break;
                    }
                }
            }
            color_order_opt.unwrap()
        };

        // get color_order of color_entity
        if let Some(color_q) = color_q_opt {
            // move all elements after color_order down one
            for i in color_order..self.color_list.len() - 1 {
                self.color_list[i] = self.color_list[i + 1];

                // update color_order in PaletteColor using color_q_opt
                if let Ok(mut color) = color_q.get_mut(self.color_list[i].unwrap()) {
                    *color.index = i as u8;
                }
            }

            self.color_list.truncate(self.color_list.len() - 1);
        }

        result
    }
}

#[derive(Resource)]
pub struct PaletteManager {
    // file entity -> file color data
    file_color_data: HashMap<Entity, FileColorData>,
    // color_entity -> file_entity
    colors: HashMap<Entity, Entity>,
}

impl Default for PaletteManager {
    fn default() -> Self {
        Self {
            file_color_data: HashMap::new(),
            colors: HashMap::new(),
        }
    }
}

impl PaletteManager {
    pub fn has_color(&self, color_entity: &Entity) -> bool {
        self.colors.contains_key(color_entity)
    }

    pub fn on_create_color(
        &mut self,
        file_entity: &Entity,
        color_entity: &Entity,
        color_index: usize,
        color_q_opt: Option<&mut Query<&mut PaletteColor>>,
    ) {
        if !self.file_color_data.contains_key(file_entity) {
            self.file_color_data
                .insert(*file_entity, FileColorData::new());
        }
        let file_color_data = self.file_color_data.get_mut(file_entity).unwrap();
        file_color_data.add_color(*color_entity, color_index, color_q_opt);

        self.colors.insert(*color_entity, *file_entity);
    }

    pub fn on_despawn_color(
        &mut self,
        color_entity: &Entity,
        color_q_opt: Option<&mut Query<&mut PaletteColor>>,
    ) {
        self.deregister_color(color_entity, color_q_opt);
    }

    pub fn deregister_color(
        &mut self,
        color_entity: &Entity,
        color_q_opt: Option<&mut Query<&mut PaletteColor>>,
    ) -> bool {
        let Some(file_entity) = self.colors.remove(color_entity) else {
            panic!("color entity not found");
        };

        let Some(file_color_data) = self.file_color_data.get_mut(&file_entity) else {
            panic!("color entity not found for file");
        };
        let output = file_color_data.remove_color(color_entity, color_q_opt);
        if file_color_data.colors.is_empty() {
            self.file_color_data.remove(&file_entity);
        }

        output
    }
}
