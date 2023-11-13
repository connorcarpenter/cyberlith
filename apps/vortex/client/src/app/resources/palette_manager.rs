use std::collections::{HashMap, HashSet};

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Resource, SystemState},
    world::{Mut, World},
};
use bevy_log::{info, warn};

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use render_egui::{
    egui,
    egui::{
        ecolor::HsvaGamma, epaint, lerp, pos2, remap_clamp, vec2, Align, Color32, Frame, Layout,
        Mesh, Rect, Response, Rgba, Sense, Shape, Stroke, Ui, Vec2,
    },
};

use vortex_proto::components::PaletteColor;

use crate::app::resources::{
    action::palette::PaletteAction, tab_manager::TabManager, toolbar::Toolbar,
};

#[derive(Resource)]
pub struct PaletteManager {
    selected_color_index: usize,
    // file entity -> color entities
    file_colors: HashMap<Entity, Vec<Option<Entity>>>,
    // color entity -> file entity
    colors: HashMap<Entity, Entity>,
    //
    resync_color_order: HashSet<Entity>,
    //
    current_color_entity: Option<Entity>,
    //
    last_old_color: Option<(Entity, Color32)>,
    last_new_color: Option<(Entity, Color32)>,
    sliding: bool,

    text_hex: String,
    text_r: String,
    text_g: String,
    text_b: String,
    text_h: String,
    text_s: String,
    text_v: String,
}

impl Default for PaletteManager {
    fn default() -> Self {
        Self {
            selected_color_index: 0,
            file_colors: HashMap::new(),
            colors: HashMap::new(),
            current_color_entity: None,
            resync_color_order: HashSet::new(),
            last_old_color: None,
            last_new_color: None,
            sliding: false,

            text_hex: String::new(),
            text_r: String::new(),
            text_g: String::new(),
            text_b: String::new(),
            text_h: String::new(),
            text_s: String::new(),
            text_v: String::new(),
        }
    }
}

impl PaletteManager {
    pub fn current_color_index(&self) -> usize {
        self.selected_color_index
    }

    pub fn has_file_colors(&self, file_entity: &Entity) -> bool {
        self.file_colors.contains_key(file_entity)
    }

    pub fn get_file_colors(&self, file_entity: &Entity) -> Option<&Vec<Option<Entity>>> {
        self.file_colors.get(file_entity)
    }

    pub(crate) fn select_color(&mut self, color_index: usize) {
        self.selected_color_index = color_index;
    }

    pub(crate) fn get_color_entity(
        &self,
        file_entity: &Entity,
        color_index: usize,
    ) -> Option<Entity> {
        let colors = self.file_colors.get(file_entity)?;
        let color_entity_opt = colors.get(color_index)?.as_ref();
        let color_entity = color_entity_opt?;
        Some(*color_entity)
    }

    pub(crate) fn get_color_count(&self, file_entity: &Entity) -> Option<usize> {
        let color_data = self.file_colors.get(file_entity)?;
        Some(color_data.len())
    }

    pub fn queue_resync_color_order(&mut self, file_entity: &Entity) {
        info!("queue_resync_color_order for entity: `{:?}`", file_entity);
        self.resync_color_order.insert(*file_entity);
    }

    pub fn track_old_color(&mut self, color_entity: Entity, old_color: Color32) {
        self.last_old_color = Some((color_entity, old_color));
    }

    pub fn track_new_color(&mut self, color_entity: Entity, new_color: Color32) {
        self.last_new_color = Some((color_entity, new_color));
    }

    pub fn flush_tracked_color(&mut self, tab_manager: &mut TabManager, world: &mut World) {
        let Some((color_entity_1, new_color)) = self.last_new_color.take() else {
            return;
        };
        let Some((color_entity_2, old_color)) = self.last_old_color.take() else {
            panic!("no old color");
        };

        if color_entity_2 != color_entity_1 {
            panic!("color entities don't match");
        }

        tab_manager.current_tab_execute_palette_action(
            world,
            self,
            PaletteAction::EditColor(color_entity_1, old_color, new_color, true),
        );

        self.track_old_color(color_entity_2, new_color);
    }

    pub fn register_color(
        &mut self,
        file_entity: Entity,
        color_entity: Entity,
        color_index: usize,
    ) {
        if !self.file_colors.contains_key(&file_entity) {
            self.file_colors.insert(file_entity, Vec::new());
        }
        let color_list = self.file_colors.get_mut(&file_entity).unwrap();

        if color_list.len() <= color_index {
            color_list.resize(color_index + 1, None);
        }

        color_list[color_index] = Some(color_entity);

        self.colors.insert(color_entity, file_entity);
    }

    pub fn deregister_color(
        &mut self,
        file_entity: &Entity,
        color_entity: &Entity,
        color_index: usize,
    ) {
        let Some(color_list) = self.file_colors.get_mut(file_entity) else {
            return;
        };
        let Some(Some(found_entity)) = color_list.get(color_index) else {
            return;
        };
        if found_entity != color_entity {
            panic!("no match");
        }
        color_list[color_index] = None;

        // remove None from the end of the list
        while let Some(None) = color_list.last() {
            color_list.pop();
        }

        color_list.truncate(color_list.len());

        if color_list.len() == 0 {
            self.file_colors.remove(file_entity);
        }

        self.colors.remove(color_entity);
    }

    pub fn resync_color_order(
        &mut self,
        client: &Client,
        color_q: &Query<(Entity, &PaletteColor)>,
    ) {
        if self.resync_color_order.is_empty() {
            return;
        }
        let resync_color_order = std::mem::take(&mut self.resync_color_order);
        for file_entity in resync_color_order {
            self.recalc_order(client, &file_entity, color_q);
        }
    }

    fn recalc_order(
        &mut self,
        client: &Client,
        file_entity: &Entity,
        color_q: &Query<(Entity, &PaletteColor)>,
    ) {
        let Some(color_list) = self.file_colors.get_mut(&file_entity) else {
            return;
        };

        let mut new_color_list = Vec::new();

        for (color_entity, color) in color_q.iter() {
            let colors_file_entity = color.owning_file_entity.get(client).unwrap();
            if colors_file_entity != *file_entity {
                continue;
            }
            let color_index = *color.index as usize;
            // resize if necessary
            if color_index >= new_color_list.len() {
                new_color_list.resize(color_index + 1, None);
            }
            if new_color_list[color_index].is_some() {
                warn!("Duplicate color order! {:?}", color_index);
            }
            new_color_list[color_index] = Some(color_entity);
        }

        for (index, color_entity_opt) in new_color_list.iter().enumerate() {
            info!("color order: {:?} -> {:?}", index, color_entity_opt);
        }

        *color_list = new_color_list;
    }

    pub fn insert_color(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        file_entity: Entity,
        color_index: usize,
        color: Color32,
    ) -> Entity {
        let mut color_component =
            PaletteColor::new(color_index as u8, color.r(), color.g(), color.b());
        color_component.owning_file_entity.set(client, &file_entity);
        let entity_id = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication(ReplicationConfig::Delegated)
            .insert(color_component)
            .id();

        // create new 2d vertex, add local components to 3d vertex
        self.register_color(file_entity, entity_id, color_index);

        entity_id
    }

    pub fn render(ui: &mut Ui, world: &mut World, file_entity: &Entity) {
        let right_rect = Self::render_right(ui, world, file_entity);
        let left_rect = Self::render_left(ui, world, file_entity);

        if ui.input(|i| i.pointer.any_click()) {
            let mouse_position = ui.input(|i| i.pointer.interact_pos().unwrap());
            let mut tab_manager = world.get_resource_mut::<TabManager>().unwrap();
            if right_rect.contains(mouse_position) || left_rect.contains(mouse_position) {
                tab_manager.set_focus(true);
            } else {
                tab_manager.set_focus(false);
            }
        }
    }

    fn render_left(ui: &mut Ui, world: &mut World, file_entity: &Entity) -> Rect {
        let mut frame = Frame::central_panel(ui.style());
        if world.get_resource::<TabManager>().unwrap().has_focus() {
            frame.stroke.color = Color32::WHITE;
            frame.stroke.width = 1.0;
        }
        let mut panel_rect: Rect = Rect::NOTHING;
        egui::CentralPanel::default()
            .frame(frame)
            .show_inside(ui, |ui| {
                world.resource_scope(|world, mut palette_manager: Mut<PaletteManager>| {
                    panel_rect = ui.available_rect_before_wrap();
                    palette_manager.render_selection_colors(ui, world, file_entity);
                });
            });
        panel_rect
    }

    fn render_right(ui: &mut Ui, world: &mut World, file_entity: &Entity) -> Rect {
        let mut panel_rect: Rect = Rect::NOTHING;
        egui::SidePanel::right("palette_right_panel")
            .resizable(true)
            .show_inside(ui, |ui| {
                panel_rect = ui.available_rect_before_wrap();
                let size = ui.available_size();
                let size = size.x.min(size.y / 3.0);

                ui.vertical_centered(|ui| {
                    Frame::none().inner_margin(4.0).show(ui, |ui| {
                        let margin = ui.style().spacing.item_spacing.x;
                        ui.allocate_ui_with_layout(
                            Vec2::new((26.0 + margin) * 4.0, 32.0),
                            Layout::top_down(Align::Center),
                            |ui| {
                                world.resource_scope(
                                    |world, mut palette_manager: Mut<PaletteManager>| {
                                        palette_manager.render_edit_buttons_impl(ui, world);
                                    },
                                );
                            },
                        );
                    });
                });
                ui.vertical_centered(|ui| {
                    Frame::none().inner_margin(4.0).show(ui, |ui| {
                        world.resource_scope(|world, mut palette_manager: Mut<PaletteManager>| {
                            palette_manager.render_edit_color_picker_impl(
                                ui,
                                world,
                                file_entity,
                                size,
                            );
                        });
                    });
                });
                ui.vertical_centered(|ui| {
                    Frame::none().inner_margin(4.0).show(ui, |ui| {
                        ui.allocate_ui_with_layout(
                            Vec2::new(size, size),
                            Layout::top_down(Align::Center),
                            |ui| {
                                world.resource_scope(
                                    |world, mut palette_manager: Mut<PaletteManager>| {
                                        palette_manager.render_edit_text_input_impl(
                                            ui,
                                            world,
                                            file_entity,
                                        );
                                    },
                                );
                            },
                        );
                    });
                });
            });
        panel_rect
    }

    fn render_selection_colors(&mut self, ui: &mut Ui, world: &mut World, file_entity: &Entity) {
        let Some(colors) = self.file_colors.get(&file_entity) else {
            return;
        };

        let mut color_q = world.query::<&PaletteColor>();

        let size = Vec2::new(32.0, 32.0);
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
                        let Ok(color_component) = color_q.get(world, *color_entity) else {
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
        let selected_color_index = self.selected_color_index;
        world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
            let current_file_entity = *tab_manager.current_tab_entity().unwrap();
            tab_manager.current_tab_execute_palette_action(
                world,
                self,
                PaletteAction::SelectColor(
                    current_file_entity,
                    color_index_picked,
                    selected_color_index,
                ),
            );
        });
    }

    fn render_edit_buttons_impl(&mut self, ui: &mut Ui, world: &mut World) {
        ui.horizontal(|ui| {
            // new color
            if Toolbar::button(ui, "➕", "New color", true).clicked() {
                palette_file_insert_color(self, world);
            }

            // delete color
            if Toolbar::button(ui, "-", "Delete color", true).clicked() {
                palette_file_delete_color(self, world);
            }

            // move frame left / right
            let current_file_entity = *world
                .get_resource::<TabManager>()
                .unwrap()
                .current_tab_entity()
                .unwrap();
            let current_color_index = self.current_color_index();
            let color_count = self
                .get_color_count(&current_file_entity)
                .unwrap_or_default();

            {
                // move frame left
                let enabled = current_color_index > 0;
                let response = Toolbar::button(ui, "⬅", "Move color left", enabled);
                if enabled && response.clicked() {
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_palette_action(
                            world,
                            self,
                            PaletteAction::MoveColor(
                                current_file_entity,
                                current_color_index,
                                current_color_index - 1,
                            ),
                        );
                    });
                }
            }

            {
                // move frame right
                let enabled = color_count > 0 && current_color_index < color_count - 1;
                let response = Toolbar::button(ui, "➡", "Move color right", enabled);
                if enabled && response.clicked() {
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_palette_action(
                            world,
                            self,
                            PaletteAction::MoveColor(
                                current_file_entity,
                                current_color_index,
                                current_color_index + 1,
                            ),
                        );
                    });
                }
            }
        });
    }

    fn render_edit_color_picker_impl(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        file_entity: &Entity,
        size: f32,
    ) {
        ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);

        let Some(color_entity) = self.get_color_entity(file_entity, self.selected_color_index) else {
            return;
        };
        let mut color_q = world.query::<&mut PaletteColor>();
        let Ok(mut color_component) = color_q.get_mut(world, color_entity) else {
            return;
        };

        let current_color =
            Color32::from_rgb(*color_component.r, *color_component.g, *color_component.b);
        let mut hsvag = HsvaGamma::from(current_color);
        let opaque = HsvaGamma { a: 1.0, ..hsvag };
        let HsvaGamma { h, s, v, a: _ } = &mut hsvag;

        let mut color_changed = false;
        if color_slider_1d(ui, size, h, |h| {
            HsvaGamma {
                h,
                s: 1.0,
                v: 1.0,
                a: 1.0,
            }
            .into()
        })
        .on_hover_text("Hue")
        .interact_pointer_pos()
        .is_some()
        {
            color_changed = true;
        }

        if color_slider_2d(ui, size, s, v, |s, v| HsvaGamma { s, v, ..opaque }.into())
            .interact_pointer_pos()
            .is_some()
        {
            color_changed = true;
        }

        if color_changed {
            let new_color: Color32 = hsvag.into();
            *color_component.r = new_color.r();
            *color_component.g = new_color.g();
            *color_component.b = new_color.b();
            self.track_new_color(color_entity, new_color);
            if !self.sliding {
                self.sliding = true;
            }
        } else {
            if self.sliding {
                // released
                self.sliding = false;

                world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                    self.flush_tracked_color(&mut tab_manager, world);
                });
            }
        }
    }

    fn render_edit_text_input_impl(
        &mut self,
        ui: &mut Ui,
        world: &mut World,
        file_entity: &Entity,
    ) {
        let Some(color_entity) = self.get_color_entity(file_entity, self.selected_color_index) else {
            return;
        };
        let mut color_q = world.query::<&mut PaletteColor>();
        let Ok(mut color_component) = color_q.get_mut(world, color_entity) else {
            return;
        };

        let current_color_rgb =
            Color32::from_rgb(*color_component.r, *color_component.g, *color_component.b);
        let mut current_color_hsv = Hsv::from(current_color_rgb);
        let mut should_flush_color = false;

        // update state
        if self.current_color_entity != Some(color_entity) {
            self.current_color_entity = Some(color_entity);
            self.text_hex = color_to_hex(current_color_rgb);
            self.text_r = color_component.r.to_string();
            self.text_g = color_component.g.to_string();
            self.text_b = color_component.b.to_string();
            self.text_h = current_color_hsv.h.to_string();
            self.text_s = current_color_hsv.s.to_string();
            self.text_v = current_color_hsv.v.to_string();
            self.track_old_color(color_entity, current_color_rgb);
        }

        // continue rendering
        ui.horizontal(|ui| {
            Frame::none().inner_margin(4.0).show(ui, |ui| {
                // label
                ui.label("Hex color:");
                // text edit
                if ui.text_edit_singleline(&mut self.text_hex).changed() {
                    if let Some(new_color) = hex_to_color(&self.text_hex) {
                        *color_component.r = new_color.r();
                        *color_component.g = new_color.g();
                        *color_component.b = new_color.b();
                    }
                }
            });
        });

        ui.vertical_centered(|ui| {
            Frame::none().inner_margin(4.0).show(ui, |ui| {
                egui::Grid::new("component-edit").show(ui, |ui| {
                    // R
                    {
                        ui.add(egui::Label::new("R"));
                        let edit_response = ui.text_edit_singleline(&mut self.text_r);
                        if edit_response.changed() {
                            // change r value
                            if let Ok(r) = self.text_r.parse::<u8>() {
                                *color_component.r = r;
                                self.track_new_color(
                                    color_entity,
                                    Color32::from_rgb(r, *color_component.g, *color_component.b),
                                );
                            }
                        }
                        if edit_response.lost_focus() {
                            should_flush_color = true;
                        }
                    }
                    // H
                    {
                        ui.add(egui::Label::new("H"));
                        let edit_response = ui.text_edit_singleline(&mut self.text_h);
                        if edit_response.changed() {
                            // change h value
                            if let Ok(h) = self.text_h.parse::<u16>() {
                                current_color_hsv.h = h;
                                let (r, g, b) = current_color_hsv.to_rgb();
                                *color_component.r = r;
                                *color_component.g = g;
                                *color_component.b = b;
                                self.track_new_color(color_entity, Color32::from_rgb(r, g, b));
                            }
                        }
                        if edit_response.lost_focus() {
                            should_flush_color = true;
                        }
                    }
                    ui.end_row();
                    // G
                    {
                        ui.add(egui::Label::new("G"));
                        let edit_response = ui.text_edit_singleline(&mut self.text_g);
                        if edit_response.changed() {
                            // change g value
                            if let Ok(g) = self.text_g.parse::<u8>() {
                                *color_component.g = g;
                                self.track_new_color(
                                    color_entity,
                                    Color32::from_rgb(*color_component.r, g, *color_component.b),
                                );
                            }
                        }
                        if edit_response.lost_focus() {
                            should_flush_color = true;
                        }
                    }
                    // S
                    {
                        ui.add(egui::Label::new("S"));
                        let edit_response = ui.text_edit_singleline(&mut self.text_s);
                        if edit_response.changed() {
                            // change s value
                            if let Ok(s) = self.text_s.parse::<u8>() {
                                current_color_hsv.s = s;
                                let (r, g, b) = current_color_hsv.to_rgb();
                                *color_component.r = r;
                                *color_component.g = g;
                                *color_component.b = b;
                                self.track_new_color(color_entity, Color32::from_rgb(r, g, b));
                            }
                        }
                        if edit_response.lost_focus() {
                            should_flush_color = true;
                        }
                    }
                    ui.end_row();
                    // B
                    {
                        ui.add(egui::Label::new("B"));
                        let edit_response = ui.text_edit_singleline(&mut self.text_b);
                        if edit_response.changed() {
                            // change b value
                            if let Ok(b) = self.text_b.parse::<u8>() {
                                *color_component.b = b;
                                self.track_new_color(
                                    color_entity,
                                    Color32::from_rgb(*color_component.r, *color_component.g, b),
                                );
                            }
                        }
                        if edit_response.lost_focus() {
                            should_flush_color = true;
                        }
                    }
                    // V
                    {
                        ui.add(egui::Label::new("V"));
                        let edit_response = ui.text_edit_singleline(&mut self.text_v);
                        if edit_response.changed() {
                            // change v value
                            if let Ok(v) = self.text_v.parse::<u8>() {
                                current_color_hsv.v = v;
                                let (r, g, b) = current_color_hsv.to_rgb();
                                *color_component.r = r;
                                *color_component.g = g;
                                *color_component.b = b;
                                self.track_new_color(color_entity, Color32::from_rgb(r, g, b));
                            }
                        }
                        if edit_response.lost_focus() {
                            should_flush_color = true;
                        }
                    }
                    ui.end_row();
                });
            });
        });

        // flush color
        if should_flush_color {
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                self.flush_tracked_color(&mut tab_manager, world);
            });
        }
    }
}

/// Number of vertices per dimension in the color sliders.
/// We need at least 6 for hues, and more for smooth 2D areas.
/// Should always be a multiple of 6 to hit the peak hues in HSV/HSL (every 60°).
const N: u32 = 6 * 6;

fn color_slider_1d(
    ui: &mut Ui,
    width: f32,
    value: &mut f32,
    color_at: impl Fn(f32) -> Color32,
) -> Response {
    let desired_size = vec2(width, 20.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

    if let Some(mpos) = response.interact_pointer_pos() {
        *value = remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);

        {
            // fill color:
            let mut mesh = Mesh::default();
            for i in 0..=N {
                let t = i as f32 / (N as f32);
                let color = color_at(t);
                let x = lerp(rect.left()..=rect.right(), t);
                mesh.colored_vertex(pos2(x, rect.top()), color);
                mesh.colored_vertex(pos2(x, rect.bottom()), color);
                if i < N {
                    mesh.add_triangle(2 * i + 0, 2 * i + 1, 2 * i + 2);
                    mesh.add_triangle(2 * i + 1, 2 * i + 2, 2 * i + 3);
                }
            }
            ui.painter().add(Shape::mesh(mesh));
        }

        ui.painter().rect_stroke(rect, 0.0, visuals.bg_stroke); // outline

        {
            // Show where the slider is at:
            let x = lerp(rect.left()..=rect.right(), *value);
            let r = rect.height() / 4.0;
            let picked_color = color_at(*value);
            ui.painter().add(Shape::convex_polygon(
                vec![
                    pos2(x, rect.center().y),   // tip
                    pos2(x + r, rect.bottom()), // right bottom
                    pos2(x - r, rect.bottom()), // left bottom
                ],
                picked_color,
                Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
            ));
        }
    }

    response
}

/// # Arguments
/// * `x_value` - X axis, either saturation or value (0.0-1.0).
/// * `y_value` - Y axis, either saturation or value (0.0-1.0).
/// * `color_at` - A function that dictates how the mix of saturation and value will be displayed in the 2d slider.
/// E.g.: `|x_value, y_value| HsvaGamma { h: 1.0, s: x_value, v: y_value, a: 1.0 }.into()` displays the colors as follows: top-left: white \[s: 0.0, v: 1.0], top-right: fully saturated color \[s: 1.0, v: 1.0], bottom-right: black \[s: 0.0, v: 1.0].
///
fn color_slider_2d(
    ui: &mut Ui,
    size: f32,
    x_value: &mut f32,
    y_value: &mut f32,
    color_at: impl Fn(f32, f32) -> Color32,
) -> Response {
    let desired_size = Vec2::splat(size);
    let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click_and_drag());

    if let Some(mpos) = response.interact_pointer_pos() {
        *x_value = remap_clamp(mpos.x, rect.left()..=rect.right(), 0.0..=1.0);
        *y_value = remap_clamp(mpos.y, rect.bottom()..=rect.top(), 0.0..=1.0);
    }

    if ui.is_rect_visible(rect) {
        let visuals = ui.style().interact(&response);
        let mut mesh = Mesh::default();

        for xi in 0..=N {
            for yi in 0..=N {
                let xt = xi as f32 / (N as f32);
                let yt = yi as f32 / (N as f32);
                let color = color_at(xt, yt);
                let x = lerp(rect.left()..=rect.right(), xt);
                let y = lerp(rect.bottom()..=rect.top(), yt);
                mesh.colored_vertex(pos2(x, y), color);

                if xi < N && yi < N {
                    let x_offset = 1;
                    let y_offset = N + 1;
                    let tl = yi * y_offset + xi;
                    mesh.add_triangle(tl, tl + x_offset, tl + y_offset);
                    mesh.add_triangle(tl + x_offset, tl + y_offset, tl + y_offset + x_offset);
                }
            }
        }
        ui.painter().add(Shape::mesh(mesh)); // fill

        ui.painter().rect_stroke(rect, 0.0, visuals.bg_stroke); // outline

        // Show where the slider is at:
        let x = lerp(rect.left()..=rect.right(), *x_value);
        let y = lerp(rect.bottom()..=rect.top(), *y_value);
        let picked_color = color_at(*x_value, *y_value);
        ui.painter().add(epaint::CircleShape {
            center: pos2(x, y),
            radius: rect.width() / 24.0,
            fill: picked_color,
            stroke: Stroke::new(visuals.fg_stroke.width, contrast_color(picked_color)),
        });
    }

    response
}

fn contrast_color(color: impl Into<Rgba>) -> Color32 {
    if color.into().intensity() < 0.5 {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}

fn color_to_hex(color: Color32) -> String {
    format!("#{:02X}{:02X}{:02X}", color.r(), color.g(), color.b())
}

fn hex_to_color(hex: &str) -> Option<Color32> {
    if hex.len() != 7 {
        return None;
    }
    let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
    let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
    let b = u8::from_str_radix(&hex[5..7], 16).ok()?;
    Some(Color32::from_rgb(r, g, b))
}

struct Hsv {
    h: u16,
    s: u8,
    v: u8,
}

impl From<Color32> for Hsv {
    fn from(color: Color32) -> Self {
        let base = HsvaGamma::from(color);
        Self {
            h: (base.h * 360.0) as u16,
            s: (base.s * 100.0) as u8,
            v: (base.v * 100.0) as u8,
        }
    }
}

impl Hsv {
    fn to_rgb(&self) -> (u8, u8, u8) {
        let h = self.h as f32 / 360.0;
        let s = self.s as f32 / 100.0;
        let v = self.v as f32 / 100.0;
        let hsvg = HsvaGamma { h, s, v, a: 1.0 };
        let rgb = Color32::from(hsvg);
        (rgb.r(), rgb.g(), rgb.b())
    }
}

pub(crate) fn palette_file_insert_color(palette_manager: &mut PaletteManager, world: &mut World) {
    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
        let current_file_entity = *tab_manager.current_tab_entity().unwrap();
        let current_color_index = palette_manager.current_color_index();

        // copy all rotations from current frame
        let stored_color;
        let current_color_entity = palette_manager
            .get_color_entity(&current_file_entity, current_color_index)
            .unwrap();
        let mut color_q = world.query::<&PaletteColor>();
        let Ok(color) = color_q.get(world, current_color_entity) else {
            return;
        };
        stored_color = Color32::from_rgb(*color.r, *color.g, *color.b);

        // execute insertion
        tab_manager.current_tab_execute_palette_action(
            world,
            palette_manager,
            PaletteAction::InsertColor(
                current_file_entity,
                current_color_index + 1,
                Some(stored_color),
            ),
        );
    });
}

pub(crate) fn palette_file_delete_color(palette_manager: &mut PaletteManager, world: &mut World) {
    let Some(current_file_entity) = world.get_resource::<TabManager>().unwrap().current_tab_entity() else {
        return;
    };
    let current_file_entity = *current_file_entity;

    let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
    let (mut commands, client) = system_state.get_mut(world);

    // delete color
    let current_color_index = palette_manager.current_color_index();
    let Some(current_frame_entity) = palette_manager.get_color_entity(&current_file_entity, current_color_index) else {
        return;
    };

    // check whether we can delete vertex
    let auth_status = commands
        .entity(current_frame_entity)
        .authority(&client)
        .unwrap();
    if !auth_status.is_granted() && !auth_status.is_available() {
        // do nothing, file is not available
        // TODO: queue for deletion? check before this?
        warn!(
            "Color `{:?}` is not available for deletion!",
            current_frame_entity
        );
        return;
    }

    let current_color_index = palette_manager.current_color_index();

    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
        tab_manager.current_tab_execute_palette_action(
            world,
            palette_manager,
            PaletteAction::DeleteColor(current_file_entity, current_color_index),
        );
    });
}
