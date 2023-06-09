use bevy_ecs::{
    entity::Entity,
    prelude::ResMut,
    system::{Commands, Query, Res, SystemState},
    world::{Mut, World},
};
use bevy_log::info;
use naia_bevy_client::{Client, CommandsExt, EntityAuthStatus};
use render_egui::{
    egui,
    egui::{
        emath, remap, vec2, Color32, Id, NumExt, Rect, Response, Rounding, Sense, Shape, Stroke,
        TextStyle, Ui, WidgetText,
    },
};
use vortex_proto::components::{ChangelistEntry, EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild};

use crate::app::{
    components::file_system::{
        ContextMenuAction, FileSystemParent, FileSystemUiState, ModalRequestType,
    },
    resources::{
        action_stack::{Action, ActionStack},
        global::Global,
    },
    ui::UiState,
};
use crate::app::components::file_system::{ChangelistContextMenuAction, ChangelistUiState};

struct RowColors {
    available: Option<Color32>,
}

const UNSELECTED_COLORS: RowColors = RowColors {
    available: None,
};
const HOVER_COLORS: RowColors = RowColors {
    available: Some(Color32::from_gray(72)),
};
const SELECTED_COLORS: RowColors = RowColors {
    available: Some(Color32::from_gray(128)),
};

pub struct ChangelistRowUiWidget;

impl ChangelistRowUiWidget {
    pub fn render_row(
        ui: &mut Ui,
        world: &mut World,
        row_entity: Entity,
    ) {
        let mut system_state: SystemState<Query<(&ChangelistEntry, &ChangelistUiState)>> = SystemState::new(world);
        let query = system_state.get(world);
        let Ok((entry, ui_state)) = query.get(row_entity) else {
            return;
        };

        let name = &*entry.name;
        let is_dir = *entry.kind == EntryKind::Directory;
        let unicode_icon = if is_dir { "📁" } else { "📃" };
        let text_str = format!("{} {}", unicode_icon, name);

        let widget_text: WidgetText = text_str.into();
        let wrap_width = ui.available_width();
        let text = widget_text.into_galley(ui, None, wrap_width, TextStyle::Button);

        let mut desired_size = text.size();
        desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);

        let (mut row_rect, row_response) = ui.allocate_at_least(desired_size, Sense::click());



        if ui.is_rect_visible(row_response.rect) {
            let item_spacing = 4.0;
            let indent_spacing = 14.0;

            let text_size = text.size();

            let mut inner_pos = ui.layout().align_size_within_rect(text_size, row_rect).min;

            // Add Margin
            inner_pos.x += 4.0;

            // Draw Row
            {
                let row_fill_colors = {
                    if ui_state.selected {
                        SELECTED_COLORS
                    } else {
                        if row_response.hovered() {
                            HOVER_COLORS
                        } else {
                            UNSELECTED_COLORS
                        }
                    }
                };

                if let Some(row_fill_color) = row_fill_colors.available {
                    row_rect.min.y -= 1.0;
                    row_rect.max.y += 2.0;
                    row_rect.max.x -= 2.0;

                    ui.painter()
                        .rect(row_rect, Rounding::none(), row_fill_color, Stroke::NONE);
                }
            }

            // spacing
            inner_pos.x += 14.0;

            // Draw Name Text
            {
                text.paint_with_visuals(ui.painter(), inner_pos, ui.style().noninteractive());
                inner_pos.x += text_size.x + item_spacing;
            }

            // Draw Path Text
            {
                let path_widget_text: WidgetText = (&*entry.path).into();
                let path_wrap_width = ui.available_width();
                let path_text = path_widget_text.into_galley(ui, None, path_wrap_width, TextStyle::Button);
                let path_text_size = path_text.size();

                path_text.paint_with_visuals(ui.painter(), inner_pos, ui.style().noninteractive());
                inner_pos.x += path_text_size.x + item_spacing;
            }
        }

        Self::handle_interactions(
            world,
            &row_entity,
            row_response,
        );
    }

    // Interactions

    pub fn handle_interactions(
        world: &mut World,
        row_entity: &Entity,
        row_response: Response,
    ) {
        let Some(mut ui_state) = world.get_mut::<ChangelistUiState>(*row_entity) else {
            return;
        };

        let left_clicked = row_response.clicked();
        let mut context_menu_response = None;

        // Right-click Context menu
        row_response.context_menu(|ui| {
            context_menu_response = Some(ChangelistContextMenuAction::None);

            if ui
                .add_enabled(true, egui::Button::new("↘ Commit"))
                .clicked()
            {
                context_menu_response = Some(ChangelistContextMenuAction::Commit);
                ui.close_menu();
            }
            if ui
                .add_enabled(true, egui::Button::new("⟲ Rollback"))
                .clicked()
            {
                context_menu_response = Some(ChangelistContextMenuAction::Rollback);
                ui.close_menu();
            }
        });
        if let Some(action) = context_menu_response {
            let just_opened = ui_state.context_menu_response.is_none();
            ui_state.context_menu_response = Some(action);
            if just_opened {
                // context menu just opened
                Self::on_row_click(world, row_entity);
            }
        } else {
            if let Some(action) = ui_state.context_menu_response.clone() {
                // context menu just closed
                ui_state.context_menu_response = None;
                match action {
                    ChangelistContextMenuAction::None => {
                        info!("just closed");
                        return;
                    }
                    ChangelistContextMenuAction::Commit => {
                        Self::on_click_commit(world, row_entity);
                        return;
                    }
                    ChangelistContextMenuAction::Rollback => {
                        Self::on_click_rollback(world, row_entity);
                        return;
                    }
                }
            }
        }

        // Left-button click
        if left_clicked {
            Self::on_row_click(world, row_entity);
            return;
        }
    }

    pub fn on_row_click(world: &mut World, row_entity: &Entity) {
        let mut system_state: SystemState<ResMut<ActionStack>> =
            SystemState::new(world);
        let mut action_stack = system_state.get_mut(world);
        let mut entities = Vec::new();
        entities.push(*row_entity);
        action_stack.buffer_action(Action::SelectEntries(entities));
    }

    pub fn on_click_commit(world: &mut World, row_entity: &Entity) {
        todo!();
    }

    pub fn on_click_rollback(world: &mut World, row_entity: &Entity) {
        todo!();
    }
}