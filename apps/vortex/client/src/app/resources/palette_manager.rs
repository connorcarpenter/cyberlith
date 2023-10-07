use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Resource, world::{World, Mut}};

use render_egui::{egui::{ecolor::HsvaGamma, lerp, Mesh, pos2, remap_clamp, Response, Rgba, Shape, Stroke, vec2, Ui, Frame, Layout, Align, Color32, Sense, Vec2}, egui};

use vortex_proto::components::PaletteColor;

#[derive(Resource)]
pub struct PaletteManager {
    selected_color_index: usize,
    // file entity -> color entities
    colors: HashMap<Entity, Vec<Option<Entity>>>,
}

impl Default for PaletteManager {
    fn default() -> Self {
        Self {
            selected_color_index: 0,
            colors: HashMap::new(),
        }
    }
}

impl PaletteManager {
    pub fn render(ui: &mut Ui, world: &mut World, file_entity: &Entity) {
        Self::render_right(ui, world, file_entity);
        Self::render_left(ui, world, file_entity);
    }

    fn render_left(ui: &mut Ui, world: &mut World, file_entity: &Entity) {
        egui::CentralPanel::default()
            .show_inside(ui, |ui| {
                Self::selection_render(ui, world, file_entity);
            });
    }

    fn render_right(ui: &mut Ui, world: &mut World, file_entity: &Entity) {
        egui::SidePanel::right("right_panel")
            .resizable(true)
            .show_inside(ui, |ui| {
                Self::edit_render(ui, world, file_entity);
                ui.allocate_space(ui.available_size());
            });
    }

    fn selection_render(ui: &mut Ui, world: &mut World, file_entity: &Entity) {
        world.resource_scope(|world, mut palette_manager: Mut<PaletteManager>| {
            palette_manager.render_selection_colors(ui, world, file_entity);
        });
    }

    fn edit_render(ui: &mut Ui, world: &mut World, file_entity: &Entity) {
        world.resource_scope(|world, mut palette_manager: Mut<PaletteManager>| {
            palette_manager.render_edit(ui, world, file_entity);
        });
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
            self.colors.remove(file_entity);
        }
    }

    fn select_color(&mut self, color_index: usize) {
        self.selected_color_index = color_index;
    }

    fn render_selection_colors(&mut self, ui: &mut Ui, world: &mut World, file_entity: &Entity) {

        let Some(colors) = self.colors.get(&file_entity) else {
            return;
        };

        let mut color_q = world.query::<&PaletteColor>();

        let size = Vec2::new(32.0, 32.0);
        let mut color_index_picked = None;

        ui.with_layout(Layout::left_to_right(Align::Min).with_main_wrap(true), |ui| {
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

                let Some(color_index_picked) = color_index_picked else {
                    return;
                };
                self.selected_color_index = color_index_picked;
            });
        });
    }

    fn get_color_entity(&self, file_entity: &Entity, color_index: usize) -> Option<Entity> {
        let colors = self.colors.get(file_entity)?;
        let color_entity_opt = colors.get(color_index)?.as_ref();
        let color_entity = color_entity_opt?;
        Some(*color_entity)
    }

    fn render_edit(&mut self, ui: &mut Ui, world: &mut World, file_entity: &Entity) {

        let Some(color_entity) = self.get_color_entity(file_entity, self.selected_color_index) else {
            return;
        };
        let mut color_q = world.query::<&mut PaletteColor>();
        let Ok(mut color_component) = color_q.get_mut(world, color_entity) else {
            return;
        };

        let current_color = Color32::from_rgb(*color_component.r, *color_component.g, *color_component.b);
        let mut hsvag = HsvaGamma::from(current_color);
        let HsvaGamma { h, s, v, a: _ } = &mut hsvag;

        if color_slider_1d(
            ui,
            h,
            |h| {
                HsvaGamma {
                    h,
                    s: 1.0,
                    v: 1.0,
                    a: 1.0,
                }
                    .into()
            })
            .on_hover_text("Hue")
            .interact_pointer_pos().is_some()
        {
            let new_color: Color32 = hsvag.into();
            *color_component.r = new_color.r();
            *color_component.g = new_color.g();
            *color_component.b = new_color.b();
        }

        //color_slider_2d(ui, s, v, |s, v| HsvaGamma { s, v, ..opaque }.into());
    }
}

/// Number of vertices per dimension in the color sliders.
/// We need at least 6 for hues, and more for smooth 2D areas.
/// Should always be a multiple of 6 to hit the peak hues in HSV/HSL (every 60°).
const N: u32 = 6 * 6;

fn color_slider_1d(ui: &mut Ui, value: &mut f32, color_at: impl Fn(f32) -> Color32) -> Response {

    let desired_size = vec2(ui.spacing().slider_width, ui.spacing().interact_size.y);
    let (rect, response) = ui.allocate_at_least(desired_size, Sense::click_and_drag());

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

fn contrast_color(color: impl Into<Rgba>) -> Color32 {
    if color.into().intensity() < 0.5 {
        Color32::WHITE
    } else {
        Color32::BLACK
    }
}