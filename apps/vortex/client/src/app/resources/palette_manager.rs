use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Resource, world::{World, Mut}};

use render_egui::{egui::{Ui, Frame, Layout, Align, Color32, Sense, Vec2}, egui};

use vortex_proto::components::PaletteColor;

#[derive(Resource)]
pub struct PaletteManager {
    // file entity -> color entities
    colors: HashMap<Entity, Vec<Option<Entity>>>,
}

impl Default for PaletteManager {
    fn default() -> Self {
        Self {
            colors: HashMap::new(),
        }
    }
}

impl PaletteManager {
    pub fn render(ui: &mut Ui, world: &mut World, file_entity: Entity) {
        Self::render_right(ui, world);
        Self::render_left(ui, world, file_entity);
    }

    fn render_left(ui: &mut Ui, world: &mut World, file_entity: Entity) {
        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                Self::selection_render(ui, world, file_entity);
            });
    }

    fn render_right(ui: &mut Ui, world: &mut World) {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .show_inside(ui, |ui| {
                Self::edit_render(ui, world);
                ui.allocate_space(ui.available_size());
            });
    }

    fn selection_render(ui: &mut Ui, world: &mut World, file_entity: Entity) {
        world.resource_scope(|world, mut palette_manager: Mut<PaletteManager>| {
            palette_manager.render_selection_colors(ui, world, file_entity);
        });
    }

    fn edit_render(ui: &mut Ui, world: &mut World) {
        ui.label("Edit");
    }

    pub fn register_color(&mut self, file_entity: Entity, color_entity: Entity, color_index: usize) {

        if !self.colors.contains_key(&file_entity) {
            self.colors.insert(file_entity, Vec::new());
        }
        let color_list = self.colors.get_mut(&file_entity).unwrap();

        if color_list.len() <= color_index {
            color_list.resize(color_index + 1, None);
        }

        color_list[color_index] = Some(color_entity);
    }

    pub fn deregister_color(&mut self, file_entity: &Entity, color_entity: &Entity, color_index: usize) {

        let Some(color_list) = self.colors.get_mut(file_entity) else {
            return;
        };
        color_list[color_index] = None;

        // remove None from the end of the list
        while let Some(None) = color_list.last() {
            color_list.pop();
        }

        color_list.truncate(color_list.len());

        if color_list.len() == 0 {
            self.colors.remove(file_entity);
        }
    }

    fn render_selection_colors(&mut self, ui: &mut Ui, world: &mut World, file_entity: Entity) {

        let Some(colors) = self.colors.get(&file_entity) else {
            return;
        };

        let mut color_q = world.query::<&PaletteColor>();

        let size = Vec2::new(32.0, 32.0);

        ui.with_layout(Layout::left_to_right(Align::Min).with_main_wrap(true), |ui| {
            Frame::none().inner_margin(8.0).show(ui, |ui| {
                for color_entity_opt in colors.iter() {
                    let Some(color_entity) = color_entity_opt else {
                        continue;
                    };
                    let Ok(color_component) = color_q.get(world, *color_entity) else {
                        continue;
                    };
                    let r = *color_component.r;
                    let g = *color_component.g;
                    let b = *color_component.b;
                    let color = Color32::from_rgb(r, g, b);

                    let (rect, response) = ui.allocate_exact_size(size, Sense::click());

                    if ui.is_rect_visible(rect) {
                        ui.painter().rect_filled(rect, 0.0, color);
                    }
                }
            });
        });
    }
}