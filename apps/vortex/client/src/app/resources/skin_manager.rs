use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::{Resource, Commands, Query, Res, SystemState}, world::World};

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use render_egui::{egui::{Ui, Vec2, Align, Color32, Frame, Layout, Sense}, egui};

use vortex_proto::components::{FaceColor, FileExtension, PaletteColor};

use crate::app::resources::{file_manager::FileManager, palette_manager::PaletteManager};

#[derive(Resource)]
pub struct SkinManager {
    // face 3d entity -> face color entity
    face_to_color_entity: HashMap<Entity, Entity>,
    // face color entity -> face 3d entity
    color_to_face_entity: HashMap<Entity, Entity>,
    //
    selected_color_index: usize,
}

impl Default for SkinManager {
    fn default() -> Self {
        Self {
            face_to_color_entity: HashMap::new(),
            color_to_face_entity: HashMap::new(),
            selected_color_index: 0,
        }
    }
}

impl SkinManager {

    pub(crate) fn selected_color_index(&self) -> usize {
        self.selected_color_index
    }

    pub(crate) fn face_to_color_entity(&self, face_3d_entity: Entity) -> Option<&Entity> {
        self.face_to_color_entity.get(&face_3d_entity)
    }

    pub(crate) fn create_networked_face_color_from_world(
        &mut self,
        world: &mut World,
        face_3d_entity: Entity,
        palette_color_entity: Entity,
    ) -> Entity {

        let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
        let (mut commands, mut client) = system_state.get_mut(world);

        let mut component = FaceColor::new();
        component.face_3d_entity.set(&client, &face_3d_entity);
        component.palette_color_entity.set(&client, &palette_color_entity);
        let face_color_entity = commands
            .spawn_empty()
            .enable_replication(&mut client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(component)
            .id();

        self.face_color_postprocess(
            face_3d_entity,
            face_color_entity,
        );

        face_color_entity
    }

    pub(crate) fn face_color_postprocess(&mut self, face_3d_entity: Entity, color_entity: Entity) {

        // TODO: change 3D face color
        // TODO: change 2D face color

        self.register_face_color(face_3d_entity, color_entity);
    }

    pub(crate) fn register_face_color(&mut self, face_3d_entity: Entity, face_color_entity: Entity) {
        self.face_to_color_entity.insert(face_3d_entity, face_color_entity);
        self.color_to_face_entity.insert(face_color_entity, face_3d_entity);
    }

    pub(crate) fn deregister_face_color(&mut self, face_color_entity: &Entity) {
        if let Some(face_3d_entity) = self.color_to_face_entity.remove(face_color_entity) {
            self.face_to_color_entity.remove(&face_3d_entity);
        }
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