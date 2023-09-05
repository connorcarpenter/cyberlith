use bevy_ecs::{
    entity::Entity,
    prelude::ResMut,
    system::{Commands, Query, SystemState},
    world::World,
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt, EntityAuthStatus};

use render_egui::{
    egui,
    egui::{NumExt, Response, Rounding, Sense, Stroke, TextStyle, Ui, WidgetText},
};
use vortex_proto::{
    channels::ChangelistActionChannel,
    components::{ChangelistEntry, ChangelistStatus, EntryKind},
    messages::{ChangelistAction, ChangelistMessage},
};

use crate::app::{
    components::file_system::ChangelistUiState,
    resources::action::{FileAction, FileActions},
    ui::widgets::colors::{
        FILE_ROW_COLORS_HOVER, FILE_ROW_COLORS_SELECTED, FILE_ROW_COLORS_UNSELECTED,
        TEXT_COLORS_HOVER, TEXT_COLORS_SELECTED, TEXT_COLORS_UNSELECTED,
    },
};

pub struct ChangelistRowUiWidget;

impl ChangelistRowUiWidget {
    pub fn render_row(ui: &mut Ui, world: &mut World, row_entity: Entity) {
        let mut system_state: SystemState<(
            Commands,
            Client,
            Query<(&ChangelistEntry, &ChangelistUiState)>,
        )> = SystemState::new(world);
        let (mut commands, client, query) = system_state.get_mut(world);

        // get auth status
        let auth_status: Option<EntityAuthStatus> = {
            if let Ok((entry, _)) = query.get(row_entity) {
                if *entry.status == ChangelistStatus::Deleted {
                    None
                } else if let Some(file_entity) = entry.file_entity.get(&client) {
                    if let Some(entity_command) = commands.get_entity(file_entity) {
                        entity_command.authority(&client)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        };
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

            let text_size = text.size();

            let mut inner_pos = ui.layout().align_size_within_rect(text_size, row_rect).min;

            // Add Margin
            inner_pos.x += 4.0;

            let (text_colors, row_fill_colors) = {
                if ui_state.selected {
                    (TEXT_COLORS_SELECTED, FILE_ROW_COLORS_SELECTED)
                } else {
                    if row_response.hovered() {
                        (TEXT_COLORS_HOVER, FILE_ROW_COLORS_HOVER)
                    } else {
                        (TEXT_COLORS_UNSELECTED, FILE_ROW_COLORS_UNSELECTED)
                    }
                }
            };

            // Draw Row
            {
                row_rect.min.y -= 1.0;
                row_rect.max.y += 2.0;
                row_rect.max.x -= 2.0;

                if let Some(text_color) = match auth_status {
                    None => match ui_state.selected {
                        true => Some(row_fill_colors.granted),
                        false => row_fill_colors.available,
                    },
                    Some(EntityAuthStatus::Available) => row_fill_colors.available,
                    Some(EntityAuthStatus::Requested) | Some(EntityAuthStatus::Releasing) => {
                        Some(row_fill_colors.requested)
                    }
                    Some(EntityAuthStatus::Granted) => Some(row_fill_colors.granted),
                    Some(EntityAuthStatus::Denied) => Some(row_fill_colors.denied),
                } {
                    ui.painter()
                        .rect(row_rect, Rounding::none(), text_color, Stroke::NONE);
                }
            }

            // spacing
            inner_pos.x += 14.0;

            // Draw Name Text
            {
                let text_color = match *entry.status {
                    ChangelistStatus::Modified => text_colors.modified,
                    ChangelistStatus::Created => text_colors.created,
                    ChangelistStatus::Deleted => text_colors.deleted,
                };
                text.paint_with_color_override(ui.painter(), inner_pos, text_color);
                inner_pos.x += text_size.x + item_spacing;
            }

            // Draw Path Text
            {
                let path_widget_text: WidgetText = (&*entry.path).into();
                let path_wrap_width = ui.available_width();
                let path_text =
                    path_widget_text.into_galley(ui, None, path_wrap_width, TextStyle::Button);
                let path_text_size = path_text.size();

                path_text.paint_with_color_override(ui.painter(), inner_pos, text_colors.disabled);
                inner_pos.x += path_text_size.x + item_spacing;
            }
        }

        Self::handle_interactions(world, &row_entity, row_response);
    }

    // Interactions

    pub fn handle_interactions(world: &mut World, row_entity: &Entity, row_response: Response) {
        let Some(mut ui_state) = world.get_mut::<ChangelistUiState>(*row_entity) else {
            return;
        };

        let left_clicked = row_response.clicked();
        let mut context_menu_response = None;

        // Right-click Context menu
        row_response.context_menu(|ui| {
            context_menu_response = Some(None);

            if ui
                .add_enabled(true, egui::Button::new("↘ Commit Change"))
                .clicked()
            {
                context_menu_response = Some(Some(ChangelistAction::Commit));
                ui.close_menu();
            }
            if ui
                .add_enabled(true, egui::Button::new("⟲ Rollback Change"))
                .clicked()
            {
                context_menu_response = Some(Some(ChangelistAction::Rollback));
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
                    None => {
                        info!("just closed");
                        return;
                    }
                    Some(ChangelistAction::Commit) => {
                        Self::on_click_commit(world, row_entity);
                        return;
                    }
                    Some(ChangelistAction::Rollback) => {
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
        let mut system_state: SystemState<(
            Commands,
            Client,
            ResMut<FileActions>,
            Query<&ChangelistEntry>,
        )> = SystemState::new(world);
        let (mut commands, client, mut file_actions, query) = system_state.get_mut(world);

        let has_auth: bool = {
            if let Ok(entry) = query.get(*row_entity) {
                if *entry.status == ChangelistStatus::Deleted {
                    true
                } else if let Some(file_entity) = entry.file_entity.get(&client) {
                    if let Some(authority) = commands.entity(file_entity).authority(&client) {
                        !authority.is_denied()
                    } else {
                        true
                    }
                } else {
                    true
                }
            } else {
                true
            }
        };

        if has_auth {
            let mut entities = Vec::new();
            entities.push(*row_entity);
            file_actions.buffer_action(FileAction::SelectEntries(entities));
        }
    }

    pub fn on_click_commit(world: &mut World, row_entity: &Entity) {
        Self::send_changelist_message(
            world,
            row_entity,
            ChangelistAction::Commit,
            Some("placeholder commit message!"),
        );
    }

    pub fn on_click_rollback(world: &mut World, row_entity: &Entity) {
        Self::send_changelist_message(world, row_entity, ChangelistAction::Rollback, None);
    }

    fn send_changelist_message(
        world: &mut World,
        row_entity: &Entity,
        action: ChangelistAction,
        opt_str: Option<&str>,
    ) {
        let mut system_state: SystemState<Client> = SystemState::new(world);
        let mut client = system_state.get_mut(world);

        let mut message = ChangelistMessage::new(action, opt_str);
        message.entity.set(&client, row_entity);

        info!("sent ChangelistMessage for entity: `{:?}`, action: {:?}", row_entity, action);
        client.send_message::<ChangelistActionChannel, ChangelistMessage>(&message);
    }
}
