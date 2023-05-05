use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, SystemState},
    world::World,
};
use naia_bevy_client::{Client, CommandsExt, EntityAuthStatus};
use render_egui::egui::{
    emath, remap, vec2, Color32, Id, NumExt, Rect, Response, Rounding, Sense, Shape, Stroke,
    TextStyle, Ui, WidgetText,
};

use crate::app::components::file_system::FileSystemUiState;

struct RowColors {
    available: Option<Color32>,
    requested: Color32,
    granted: Color32,
    denied: Color32,
}

const UNSELECTED_COLORS: RowColors = RowColors {
    available: None,
    requested: Color32::from_rgb(0, 64, 0),
    granted: Color32::from_rgb(0, 48, 64),
    denied: Color32::from_rgb(64, 0, 0),
};
const HOVER_COLORS: RowColors = RowColors {
    available: Some(Color32::from_gray(72)),
    requested: Color32::from_rgb(0, 72, 0),
    granted: Color32::from_rgb(0, 72, 96),
    denied: Color32::from_rgb(96, 0, 0),
};
const SELECTED_COLORS: RowColors = RowColors {
    available: Some(Color32::from_gray(128)),
    requested: Color32::from_rgb(0, 96, 0),
    granted: Color32::from_rgb(0, 96, 128),
    denied: Color32::from_rgb(128, 0, 0),
};

pub struct FileTreeRowUiWidget;

impl FileTreeRowUiWidget {
    pub fn render_row(
        ui: &mut Ui,
        world: &mut World,
        row_entity: &Entity,
        path: &str,
        name: &str,
        depth: usize,
        is_dir: bool,
    ) {
        let icon_fn = if is_dir {
            paint_default_icon
        } else {
            paint_no_icon
        };
        let separator = if path.len() > 0 { ":" } else { "" };
        let full_path = format!("{}{}{}", path, separator, name);
        let unicode_icon = if is_dir { "📁" } else { "📃" };
        let text_str = format!("{} {}", unicode_icon, name);

        let widget_text: WidgetText = text_str.into();
        let wrap_width = ui.available_width();
        let text = widget_text.into_galley(ui, None, wrap_width, TextStyle::Button);

        let mut expander_clicked = false;

        let mut desired_size = text.size();
        desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);

        let (mut row_rect, row_response) = ui.allocate_at_least(desired_size, Sense::click());

        let mut system_state: SystemState<(Commands, Client, Query<&FileSystemUiState>)> =
            SystemState::new(world);
        let (mut commands, client, fs_query) = system_state.get_mut(world);
        let Ok(ui_state) = fs_query.get(*row_entity) else {
            return;
        };

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
                        expander_clicked = true;
                    }
                }

                big_icon_response
            };

            // Draw Row
            {
                let row_fill_colors = {
                    if ui_state.selected {
                        SELECTED_COLORS
                    } else {
                        if row_response.hovered() || icon_response.hovered() {
                            HOVER_COLORS
                        } else {
                            UNSELECTED_COLORS
                        }
                    }
                };
                let auth_status = commands.entity(*row_entity).authority(&client);
                let row_fill_color_opt = match auth_status {
                    None | Some(EntityAuthStatus::Available) => row_fill_colors.available,
                    Some(EntityAuthStatus::Requested) => Some(row_fill_colors.requested),
                    Some(EntityAuthStatus::Granted) => Some(row_fill_colors.granted),
                    Some(EntityAuthStatus::Denied) => Some(row_fill_colors.denied),
                };

                if let Some(row_fill_color) = row_fill_color_opt {
                    row_rect.min.y -= 1.0;
                    row_rect.max.y += 2.0;
                    row_rect.max.x -= 2.0;

                    ui.painter()
                        .rect(row_rect, Rounding::none(), row_fill_color, Stroke::NONE);
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

        // Respond to click event, if not root dir
        if depth > 0 {
            if row_response.clicked() {
                Self::on_row_click(world, row_entity);
            }
        }

        // Respond to expander click event
        if expander_clicked {
            Self::on_expander_click(world, row_entity);
        }
    }

    pub fn on_row_click(world: &mut World, row_entity: &Entity) {
        let mut system_state: SystemState<(
            Commands,
            Client,
            Query<(Entity, &mut FileSystemUiState)>,
        )> = SystemState::new(world);
        let (mut commands, mut client, mut fs_query) = system_state.get_mut(world);
        for (item_entity, mut ui_state) in fs_query.iter_mut() {
            if *row_entity == item_entity {
                let was_selected = ui_state.selected;
                ui_state.selected = true;

                if !was_selected {
                    // Request Entity Authority
                    commands.entity(item_entity).request_authority(&mut client);
                }
            } else {
                let was_selected = ui_state.selected;
                ui_state.selected = false;

                // TODO: when shift/control is pressed, select multiple items

                if was_selected {
                    // Release Entity Authority
                    commands.entity(item_entity).release_authority(&mut client);
                }
            }
        }
    }

    pub fn on_expander_click(world: &mut World, row_entity: &Entity) {
        if let Some(mut ui_state) = world.get_mut::<FileSystemUiState>(*row_entity) {
            ui_state.opened = !ui_state.opened;
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
