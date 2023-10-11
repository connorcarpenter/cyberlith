use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::{Resource, Query, Res, SystemState}, world::World};

use render_egui::{egui::{Ui, Vec2, Align, Color32, Frame, Layout, Sense}, egui};

use vortex_proto::components::{FileExtension, PaletteColor};

use crate::app::resources::{file_manager::FileManager, palette_manager::PaletteManager};

#[derive(Resource)]
pub struct SkinManager {
    face_colors: HashMap<Entity, Entity>,
    selected_color_index: usize,
}

impl Default for SkinManager {
    fn default() -> Self {
        Self {
            face_colors: HashMap::new(),
            selected_color_index: 0,
        }
    }
}

impl SkinManager {
    pub(crate) fn get_face_color(&self, face_3d_entity: Entity) -> Option<&Entity> {
        self.face_colors.get(&face_3d_entity)
    }

    pub(crate) fn render_sidebar(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        current_file_entity: &Entity
    ) {
        egui::SidePanel::right("skin_right_panel")
            .resizable(true)
            .show_inside(ui, |ui| {

                let mut system_state: SystemState<(
                    Res<FileManager>,
                    Res<PaletteManager>,
                    Query<&PaletteColor>,
                )> = SystemState::new(world);
                let (file_manager, palette_manager, palette_color_q) = system_state.get_mut(world);

                let Some(palette_file_entity) = file_manager.file_get_dependency(
                    current_file_entity,
                    FileExtension::Palette,
                ) else {
                    panic!("Expected palette file dependency");
                };
                let Some(colors) = palette_manager.get_file_colors(&palette_file_entity) else {
                    return;
                };

                let size = Vec2::new(16.0, 16.0);
                let mut color_index_picked = None;

                ui.with_layout(
                    Layout::left_to_right(Align::Min).with_main_wrap(true),
                    |ui| {
                        Frame::none().inner_margin(8.0).show(ui, |ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);
                            for (color_index, color_entity_opt) in colors.iter().enumerate() {
                                let Some(color_entity) = color_entity_opt else {
                                    continue;
                                };
                                let Ok(color_component) = palette_color_q.get(*color_entity) else {
                                    continue;
                                };
                                let r = *color_component.r;
                                let g = *color_component.g;
                                let b = *color_component.b;
                                let color = Color32::from_rgb(r, g, b);

                                let (mut rect, response) = ui.allocate_exact_size(size, Sense::click());
                                if response.hovered() {
                                    rect = rect.expand(2.0);
                                }

                                if ui.is_rect_visible(rect) {
                                    ui.painter().rect_filled(rect, 0.0, color);
                                    if color_index == self.selected_color_index {
                                        rect = rect.expand(2.0);
                                        ui.painter().rect_stroke(rect, 0.0, (2.0, Color32::WHITE));
                                    } else if response.clicked() {
                                        color_index_picked = Some(color_index);
                                    }
                                }
                            }
                        });
                    },
                );

                let Some(color_index_picked) = color_index_picked else {
                    return;
                };
                if color_index_picked == self.selected_color_index {
                    return;
                }
                self.selected_color_index = color_index_picked;
            });
    }
}