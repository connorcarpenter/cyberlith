use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    event::EventWriter,
    system::{Commands, Query, Res, Resource, SystemState},
    world::World,
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use render_egui::{
    egui,
    egui::{Align, Color32, Frame, Layout, PointerButton, Sense, Ui, Vec2},
};

use vortex_proto::components::{BackgroundSkinColor, FaceColor, FileExtension, PaletteColor};

use crate::app::{
    events::ShapeColorResyncEvent,
    resources::{
        action::skin::SkinAction, file_manager::FileManager, input_manager::InputManager,
        palette_manager::PaletteManager, shape_data::CanvasShape,
    },
};

#[derive(Resource)]
pub struct SkinManager {
    // face 3d entity -> face color entity
    face_to_color_entity: HashMap<Entity, Entity>,
    // face color entity -> face 3d entity
    color_to_face_entity: HashMap<Entity, Entity>,
    // skin file entity -> bckg color entity
    file_to_bckg_entity: HashMap<Entity, Entity>,
    // bckg color entity -> skin file entity
    bckg_to_file_entity: HashMap<Entity, Entity>,

    selected_color_index: usize,
}

impl Default for SkinManager {
    fn default() -> Self {
        Self {
            face_to_color_entity: HashMap::new(),
            color_to_face_entity: HashMap::new(),
            file_to_bckg_entity: HashMap::new(),
            bckg_to_file_entity: HashMap::new(),
            selected_color_index: 0,
        }
    }
}

impl SkinManager {
    pub(crate) fn selected_color_index(&self) -> usize {
        self.selected_color_index
    }

    pub(crate) fn background_color_index(
        &self,
        client: &Client,
        file_entity: &Entity,
        bck_color_q: &Query<&BackgroundSkinColor>,
        palette_q: &Query<&PaletteColor>
    ) -> usize {
        if let Some(bckg_color_entity) = self.file_to_bckg_entity.get(file_entity) {
            if let Ok(bck_color) = bck_color_q.get(*bckg_color_entity) {
                if let Some(palette_color_entity) = bck_color.palette_color_entity.get(client) {
                    if let Ok(palette_color) = palette_q.get(palette_color_entity) {
                        return *palette_color.index as usize;
                    }
                }
            }
        }

        0
    }

    pub(crate) fn set_background_color_index(
        &self,
        client: &Client,
        palette_manager: &PaletteManager,
        file_entity: &Entity,
        back_color_q: &mut Query<&mut BackgroundSkinColor>,
        new_index: usize,
    ) {
        if let Some(bckg_color_entity) = self.file_to_bckg_entity.get(file_entity) {
            if let Ok(mut bck_color) = back_color_q.get_mut(*bckg_color_entity) {
                let Some(new_palette_color_entity) = palette_manager.get_color_entity(
                    file_entity,
                    new_index,
                ) else {
                  panic!("expected palette color entity");
                };
                bck_color.palette_color_entity.set(client, &new_palette_color_entity);
            }
        }
    }

    pub(crate) fn entity_is_face_color(&self, face_color_entity: &Entity) -> bool {
        self.color_to_face_entity.contains_key(face_color_entity)
    }

    pub(crate) fn entity_is_bckg_color(&self, bckg_color_entity: &Entity) -> bool {
        self.bckg_to_file_entity.contains_key(bckg_color_entity)
    }

    pub(crate) fn face_to_color_entity(&self, face_3d_entity: &Entity) -> Option<&Entity> {
        self.face_to_color_entity.get(face_3d_entity)
    }

    pub(crate) fn file_to_bckg_entity(&self, file_entity: &Entity) -> Option<&Entity> {
        self.file_to_bckg_entity.get(file_entity)
    }

    pub(crate) fn create_networked_face_color_from_world(
        &mut self,
        world: &mut World,
        skin_file_entity: Entity,
        face_3d_entity: Entity,
        palette_color_entity: Entity,
    ) -> Entity {
        info!("creating networked face color!");
        let mut system_state: SystemState<(Commands, Client, EventWriter<ShapeColorResyncEvent>)> =
            SystemState::new(world);
        let (mut commands, mut client, mut shape_color_resync_events) = system_state.get_mut(world);

        let mut component = FaceColor::new();
        component.skin_file_entity.set(&client, &skin_file_entity);
        component.face_3d_entity.set(&client, &face_3d_entity);
        component
            .palette_color_entity
            .set(&client, &palette_color_entity);
        let face_color_entity = commands
            .spawn_empty()
            .enable_replication(&mut client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(component)
            .id();

        self.face_color_postprocess(
            face_3d_entity,
            face_color_entity,
            &mut shape_color_resync_events,
        );

        system_state.apply(world);

        face_color_entity
    }

    pub(crate) fn bckg_color_postprocess(
        &mut self,
        file_entity: Entity,
        face_color_entity: Entity,
        shape_color_resync_events: &mut EventWriter<ShapeColorResyncEvent>,
    ) {
        shape_color_resync_events.send(ShapeColorResyncEvent);

        // register
        self.register_bckg_color(file_entity, face_color_entity);
    }

    pub(crate) fn face_color_postprocess(
        &mut self,
        face_3d_entity: Entity,
        face_color_entity: Entity,
        shape_color_resync_events: &mut EventWriter<ShapeColorResyncEvent>,
    ) {
        shape_color_resync_events.send(ShapeColorResyncEvent);

        // register
        self.register_face_color(face_3d_entity, face_color_entity);
    }

    pub(crate) fn register_bckg_color(
        &mut self,
        file_entity: Entity,
        bckg_color_entity: Entity,
    ) {
        self.file_to_bckg_entity
            .insert(file_entity, bckg_color_entity);
        self.bckg_to_file_entity
            .insert(bckg_color_entity, file_entity);
    }

    pub(crate) fn register_face_color(
        &mut self,
        face_3d_entity: Entity,
        face_color_entity: Entity,
    ) {
        self.face_to_color_entity
            .insert(face_3d_entity, face_color_entity);
        self.color_to_face_entity
            .insert(face_color_entity, face_3d_entity);
    }

    pub(crate) fn deregister_face_color(&mut self, face_color_entity: &Entity) {
        if let Some(face_3d_entity) = self.color_to_face_entity.remove(face_color_entity) {
            self.face_to_color_entity.remove(&face_3d_entity);
        }
    }

    pub(crate) fn deregister_bckg_color(&mut self, bckg_color_entity: &Entity) {
        if let Some(file_entity) = self.bckg_to_file_entity.remove(bckg_color_entity) {
            self.file_to_bckg_entity.remove(&file_entity);
        }
    }

    pub(crate) fn render_sidebar(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        current_file_entity: &Entity,
    ) -> Option<SkinAction> {
        let mut color_index_picked = None;

        let mut system_state: SystemState<(
            Client,
            Res<FileManager>,
            Res<PaletteManager>,
            Query<&BackgroundSkinColor>,
            Query<&PaletteColor>,
        )> = SystemState::new(world);
        let (
            client,
            file_manager,
            palette_manager,
            bckg_color_q,
            palette_color_q
        ) = system_state.get_mut(world);

        let Some(palette_file_entity) = file_manager.file_get_dependency(
            current_file_entity,
            FileExtension::Palette,
        ) else {
            panic!("Expected palette file dependency");
        };
        let Some(colors) = palette_manager.get_file_colors(&palette_file_entity) else {
            return None;
        };
        let bckg_color_index = self.background_color_index(
            &client,
            current_file_entity,
            &bckg_color_q,
            &palette_color_q
        );

        egui::SidePanel::right("skin_right_panel")
            .exact_width(8.0*2.0 + 48.0*2.0 + 2.0 + 10.0*2.0)
            .resizable(false)
            .show_inside(ui, |ui| {

                let size = Vec2::new(48.0, 48.0);

                ui.horizontal_top(|ui| {
                    Frame::none().inner_margin(8.0).show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);

                        for color_index in [self.selected_color_index, bckg_color_index].iter() {
                            let color_entity_opt = colors.get(*color_index).unwrap();
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

                            let (mut rect, _response) =
                                ui.allocate_exact_size(size, Sense::click());

                            if ui.is_rect_visible(rect) {
                                ui.painter().rect_filled(rect, 0.0, color);
                                rect = rect.expand(2.0);
                                ui.painter().rect_stroke(rect, 0.0, (2.0, Color32::WHITE));
                            }
                        }
                    });
                });

                ui.separator();

                let size = Vec2::new(16.0, 16.0);

                ui.with_layout(
                    Layout::left_to_right(Align::Min).with_main_wrap(true),
                    |ui| {
                        Frame::none().inner_margin(8.0).show(ui, |ui| {
                            ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);
                            for (color_index, color_entity_opt) in colors.iter().enumerate() {
                                let Some(palette_color_entity) = color_entity_opt else {
                                    continue;
                                };
                                let Ok(color_component) = palette_color_q.get(*palette_color_entity) else {
                                    continue;
                                };
                                let r = *color_component.r;
                                let g = *color_component.g;
                                let b = *color_component.b;
                                let color = Color32::from_rgb(r, g, b);

                                let (mut rect, response) =
                                    ui.allocate_exact_size(size, Sense::click());
                                if response.hovered() {
                                    rect = rect.expand(2.0);
                                }

                                if ui.is_rect_visible(rect) {
                                    ui.painter().rect_filled(rect, 0.0, color);
                                    if color_index == self.selected_color_index {
                                        rect = rect.expand(2.0);
                                        ui.painter().rect_stroke(rect, 0.0, (2.0, Color32::WHITE));
                                    } else if response.clicked_by(PointerButton::Primary) {
                                        color_index_picked = Some((color_index, *palette_color_entity, PointerButton::Primary));
                                    }
                                    if response.clicked_by(PointerButton::Secondary) {
                                        color_index_picked = Some((color_index, *palette_color_entity, PointerButton::Secondary));
                                    }
                                }
                            }
                        });
                    });
                return;
            });

        let Some((color_index_picked, palette_color_entity, click_type)) = color_index_picked else {
            return None;
        };
        match click_type {
            PointerButton::Primary => {
                if color_index_picked == self.selected_color_index {
                    return None;
                }
                self.selected_color_index = color_index_picked;

                let selected_shape = world
                    .get_resource::<InputManager>()
                    .unwrap()
                    .selected_shape_2d();
                if selected_shape.is_some() {
                    let Some((face_2d_entity, CanvasShape::Face)) = selected_shape else {
                        panic!("expected face entity");
                    };
                    return Some(SkinAction::EditColor(
                        face_2d_entity,
                        Some(palette_color_entity),
                    ));
                }
            }
            PointerButton::Secondary => {
                if color_index_picked == bckg_color_index {
                    return None;
                }

                let mut system_state: SystemState<(
                    Client,
                    EventWriter<ShapeColorResyncEvent>,
                    Res<PaletteManager>,
                    Query<&mut BackgroundSkinColor>,
                )> = SystemState::new(world);
                let (client, mut shape_color_resync_events, palette_manager, mut bckg_color_q) = system_state.get_mut(world);
                shape_color_resync_events.send(ShapeColorResyncEvent);

                self.set_background_color_index(
                    &client,
                    &palette_manager,
                    current_file_entity,
                    &mut bckg_color_q,
                    color_index_picked,)
            }
            _ => {}
        }

        return None;
    }
}
