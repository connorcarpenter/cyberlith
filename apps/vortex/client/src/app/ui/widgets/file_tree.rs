use crate::app::components::file_system::{FileSystemParent, FileSystemUiState};
use crate::app::resources::global::Global;
use bevy_ecs::{entity::Entity, world::World};
use render_egui::egui::{
    emath, remap, vec2, Align, Color32, Id, Layout, NumExt, Rect, Response, Rounding, Sense, Shape,
    Stroke, TextStyle, Ui, WidgetText,
};
use vortex_proto::components::{EntryKind, FileSystemEntry};

pub struct FileTreeUiWidget;

impl FileTreeUiWidget {
    pub fn render_root(ui: &mut Ui, world: &mut World) {
        let root_entity = world.get_resource::<Global>().unwrap().project_root_entity;
        let entry = world.entity(root_entity).get::<FileSystemEntry>().unwrap();
        let name = (*entry.name).clone();

        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            Self::render(ui, world, &root_entity, "", &name, 0);
        });
    }

    fn render(
        ui: &mut Ui,
        world: &mut World,
        entity: &Entity,
        path: &str,
        name: &str,
        depth: usize,
    ) {
        let is_directory =
            *(world.entity(*entity).get::<FileSystemEntry>().unwrap().kind) == EntryKind::Directory;

        if is_directory {
            Self::render_row(
                ui,
                world,
                entity,
                path,
                name,
                depth,
                true,
                paint_default_icon,
            );
            let opened = world
                .entity(*entity)
                .get::<FileSystemUiState>()
                .unwrap()
                .opened;
            if opened {
                Self::render_children(ui, world, entity, path, name, depth);
            }
        } else {
            Self::render_row(ui, world, entity, path, name, depth, false, paint_no_icon);
        }
    }

    fn render_children(
        ui: &mut Ui,
        world: &mut World,
        entity: &Entity,
        path: &str,
        name: &str,
        depth: usize,
    ) {
        let separator = if path.len() > 0 { ":" } else { "" };
        let full_path = format!("{}{}{}", path, separator, name);

        let parent = world.entity(*entity).get::<FileSystemParent>().unwrap();

        for child_entity in parent.get_children() {
            let child_name =
                (*(world.entity(child_entity).get::<FileSystemEntry>().unwrap()).name).clone();
            Self::render(ui, world, &child_entity, &full_path, &child_name, depth + 1);
        }
    }

    pub fn render_row(
        ui: &mut Ui,
        world: &mut World,
        entity: &Entity,
        path: &str,
        name: &str,
        depth: usize,
        is_dir: bool,
        icon_fn: impl FnOnce(&mut Ui, bool, &Response) + 'static,
    ) {
        let separator = if path.len() > 0 { ":" } else { "" };
        let full_path = format!("{}{}{}", path, separator, name);
        let unicode_icon = if is_dir { "📁" } else { "📃" };
        let text_str = format!("{} {}", unicode_icon, name);

        let widget_text: WidgetText = text_str.into();
        let wrap_width = ui.available_width();
        let text = widget_text.into_galley(ui, None, wrap_width, TextStyle::Button);

        let mut desired_size = text.size();
        desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);

        let (mut row_rect, row_response) = ui.allocate_at_least(desired_size, Sense::click());

        let mut entity_mut = world.entity_mut(*entity);
        let mut ui_state = entity_mut.get_mut::<FileSystemUiState>().unwrap();

        if ui.is_rect_visible(row_response.rect) {
            let item_spacing = 4.0;
            let indent_spacing = 14.0;

            let text_size = text.size();

            let mut inner_pos = ui.layout().align_size_within_rect(text_size, row_rect).min;

            // Add Margin
            inner_pos.x += (depth as f32 * indent_spacing) + 4.0;

            let icon_response = {
                let icon_size = vec2(ui.spacing().icon_width, ui.spacing().icon_width);
                let icon_rect = Rect::from_min_size(inner_pos, icon_size);

                let big_icon_response = ui.interact(icon_rect, Id::new(full_path), Sense::click());

                if is_dir {
                    if big_icon_response.clicked() {
                        ui_state.opened = !ui_state.opened;
                    }
                }

                big_icon_response
            };

            // Draw Row
            {
                let row_fill = if ui_state.selected {
                    Some(Color32::from_rgb(0, 92, 128))
                } else {
                    if row_response.hovered() || icon_response.hovered() {
                        Some(Color32::from_gray(70))
                    } else {
                        None
                    }
                };

                if let Some(fill_color) = row_fill {
                    row_rect.min.y -= 1.0;
                    row_rect.max.y += 2.0;
                    row_rect.max.x -= 2.0;

                    ui.painter()
                        .rect(row_rect, Rounding::none(), fill_color, Stroke::NONE);
                }
            }

            // Draw Icon
            if is_dir {
                let (small_icon_rect, _) = ui.spacing().icon_rectangles(icon_response.rect);
                let small_icon_response = icon_response.clone().with_new_rect(small_icon_rect);

                icon_fn(ui, ui_state.opened, &small_icon_response);
                inner_pos.x += small_icon_response.rect.width() + item_spacing;
            } else {
                inner_pos.x += 14.0;
            }

            // Draw Text
            {
                text.paint_with_visuals(ui.painter(), inner_pos, ui.style().noninteractive());
                inner_pos.x += text_size.x + item_spacing;
            }
        }

        if row_response.clicked() {
            ui_state.selected = !ui_state.selected;
        }
    }
}

// ----------------------------------------------------------------------------

/// Paint the arrow icon that indicated if the region is open or not
fn paint_default_icon(ui: &mut Ui, openned: bool, response: &Response) {
    let openness = if openned { 1.0 } else { 0.0 };

    let visuals = ui.style().interact(response);

    let rect = response.rect;

    // Draw a pointy triangle arrow:
    let rect = Rect::from_center_size(rect.center(), vec2(rect.width(), rect.height()) * 0.75);
    let rect = rect.expand(visuals.expansion);
    let mut points = vec![rect.left_top(), rect.right_top(), rect.center_bottom()];
    use std::f32::consts::TAU;
    let rotation = emath::Rot2::from_angle(remap(openness, 0.0..=1.0, -TAU / 4.0..=0.0));
    for p in &mut points {
        *p = rect.center() + rotation * (*p - rect.center());
    }

    ui.painter().add(Shape::convex_polygon(
        points,
        visuals.fg_stroke.color,
        Stroke::NONE,
    ));
}

pub fn paint_no_icon(_ui: &mut Ui, _openness: bool, _response: &Response) {}
