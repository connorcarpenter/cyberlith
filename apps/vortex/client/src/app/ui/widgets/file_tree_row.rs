use bevy_ecs::{
    entity::Entity,
    prelude::ResMut,
    system::{Commands, Query, SystemState},
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
use vortex_proto::components::{EntryKind, FileSystemChild, FileSystemEntry, FileSystemRootChild};

use crate::app::{
    components::file_system::{ContextMenuAction, FileSystemUiState, ModalRequestType},
    resources::action_stack::{Action, ActionStack},
    ui::UiState,
};

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
            Self::paint_default_icon
        } else {
            Self::paint_no_icon
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

        let mut system_state: SystemState<(Commands, Client, Query<&mut FileSystemUiState>)> =
            SystemState::new(world);
        let (mut commands, client, fs_query) = system_state.get_mut(world);
        let auth_status = commands
            .entity(*row_entity)
            .authority(&client);
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
                let row_fill_color_opt = match auth_status {
                    None | Some(EntityAuthStatus::Available) => row_fill_colors.available,
                    Some(EntityAuthStatus::Requested) | Some(EntityAuthStatus::Releasing) => {
                        Some(row_fill_colors.requested)
                    }
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

        Self::handle_modal_responses(depth, world, row_entity);
        Self::handle_interactions(
            depth,
            world,
            row_entity,
            auth_status,
            expander_clicked,
            row_response,
        );
    }

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

    // Interactions

    pub fn handle_interactions(
        depth: usize,
        world: &mut World,
        row_entity: &Entity,
        auth_status: Option<EntityAuthStatus>,
        expander_clicked: bool,
        row_response: Response,
    ) {
        // Respond to expander click event
        if expander_clicked {
            Self::on_expander_click(world, row_entity);
            return;
        }

        // If Root Dir, exit early
        if depth == 0 {
            return;
        }

        let Some(mut ui_state) = world.get_mut::<FileSystemUiState>(*row_entity) else {
            return;
        };

        let left_clicked = row_response.clicked();
        let mut context_menu_response = None;

        // Right-click Context menu
        row_response.context_menu(|ui| {
            context_menu_response = Some(ContextMenuAction::None);

            let can_mutate = auth_status == Some(EntityAuthStatus::Granted);

            if ui
                .add_enabled(true, egui::Button::new("📃 New File"))
                .clicked()
            {
                context_menu_response = Some(ContextMenuAction::NewFile);
                ui.close_menu();
            }
            if ui
                .add_enabled(true, egui::Button::new("📁 New Directory"))
                .clicked()
            {
                context_menu_response = Some(ContextMenuAction::NewDirectory);
                ui.close_menu();
            }
            if ui
                .add_enabled(can_mutate, egui::Button::new("✏ Rename"))
                .clicked()
            {
                context_menu_response = Some(ContextMenuAction::Rename);
                ui.close_menu();
            }
            if ui
                .add_enabled(can_mutate, egui::Button::new("🗑 Delete"))
                .clicked()
            {
                context_menu_response = Some(ContextMenuAction::Delete);
                ui.close_menu();
            }
            if ui
                .add_enabled(can_mutate, egui::Button::new("✂ Cut"))
                .clicked()
            {
                context_menu_response = Some(ContextMenuAction::Cut);
                ui.close_menu();
            }
            if ui.add_enabled(true, egui::Button::new("📷 Copy")).clicked() {
                context_menu_response = Some(ContextMenuAction::Copy);
                ui.close_menu();
            }
            if ui
                .add_enabled(true, egui::Button::new("📋 Paste"))
                .clicked()
            {
                context_menu_response = Some(ContextMenuAction::Paste);
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
                    ContextMenuAction::None => {
                        info!("just closed");
                        return;
                    }
                    ContextMenuAction::NewFile => {
                        Self::on_click_new_file(world, row_entity);
                        return;
                    }
                    ContextMenuAction::NewDirectory => {
                        info!("New Directory");
                        return;
                    }
                    ContextMenuAction::Rename => {
                        Self::on_click_rename(world, row_entity);
                        return;
                    }
                    ContextMenuAction::Delete => {
                        Self::on_click_delete(world, row_entity);
                        return;
                    }
                    ContextMenuAction::Cut => {
                        info!("Cut");
                        return;
                    }
                    ContextMenuAction::Copy => {
                        info!("Copy");
                        return;
                    }
                    ContextMenuAction::Paste => {
                        info!("Paste");
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
        let mut system_state: SystemState<(Commands, Client, ResMut<ActionStack>)> =
            SystemState::new(world);
        let (mut commands, client, mut action_stack) = system_state.get_mut(world);
        if let Some(authority) = commands.entity(*row_entity).authority(&client) {
            if authority.is_available() {
                let mut entities = Vec::new();
                entities.push(*row_entity);
                action_stack.buffer_action(Action::SelectFiles(entities));
            }
        }
    }

    pub fn on_expander_click(world: &mut World, row_entity: &Entity) {
        if let Some(mut ui_state) = world.get_mut::<FileSystemUiState>(*row_entity) {
            ui_state.opened = !ui_state.opened;
        }
    }

    pub fn on_click_new_file(world: &mut World, row_entity: &Entity) {
        world.resource_scope(|world, mut ui_state: Mut<UiState>| {

            let mut system_state: SystemState<(Client, Query<(&FileSystemEntry, Option<&FileSystemChild>, Option<&FileSystemRootChild>, &mut FileSystemUiState)>)> =
                SystemState::new(world);
            let (client, mut fs_query) = system_state.get_mut(world);
            let Ok((entry, dir_child_opt, root_child_opt, mut entry_ui_state)) = fs_query.get_mut(*row_entity) else {
                return;
            };

            let directory_entity_opt: Option<Entity> = match *entry.kind {
                EntryKind::Directory => {
                    Some(row_entity.clone())
                }
                EntryKind::File => {
                    if let Some(dir_child) = dir_child_opt {
                        Some(dir_child.parent_id.get(&client).unwrap().clone())
                    } else if let Some(_root_child) = root_child_opt {
                        None
                    } else {
                        panic!("File entry has no parent");
                    }
                }
            };

            let Some(request_handle) = ui_state.text_input_modal.open(
                "New File",
                "Create new file with name:",
                Some("file.txt"),
                "Submit"
            ) else {
                return;
            };

            entry_ui_state.modal_request = Some((ModalRequestType::NewFile(directory_entity_opt), request_handle));
        });
    }

    pub fn on_click_delete(world: &mut World, row_entity: &Entity) {
        world.resource_scope(|world, mut ui_state: Mut<UiState>| {
            let mut system_state: SystemState<Query<(&FileSystemEntry, &mut FileSystemUiState)>> =
                SystemState::new(world);
            let mut fs_query = system_state.get_mut(world);
            let Ok((entry, mut entry_ui_state)) = fs_query.get_mut(*row_entity) else {
                return;
            };

            let file_name: &str = &*entry.name;

            let Some(request_handle) = ui_state.text_input_modal.open(
                "Delete File",
                &format!("Are you sure you want to delete `{}` ?", file_name),
                None,
                "Delete"
            ) else {
                return;
            };

            entry_ui_state.modal_request =
                Some((ModalRequestType::Delete(*row_entity), request_handle));
        });
    }

    pub fn on_click_rename(world: &mut World, row_entity: &Entity) {
        world.resource_scope(|world, mut ui_state: Mut<UiState>| {
            let mut system_state: SystemState<Query<(&FileSystemEntry, &mut FileSystemUiState)>> =
                SystemState::new(world);
            let mut fs_query = system_state.get_mut(world);
            let Ok((entry, mut entry_ui_state)) = fs_query.get_mut(*row_entity) else {
                return;
            };

            let file_name: &str = &*entry.name;

            let Some(request_handle) = ui_state.text_input_modal.open(
                "Rename",
                &format!("Rename file `{}` to:", file_name),
                Some(file_name),
                "Submit"
            ) else {
                return;
            };

            entry_ui_state.modal_request = Some((ModalRequestType::Rename, request_handle));
        });
    }

    pub fn handle_modal_responses(depth: usize, world: &mut World, row_entity: &Entity) {
        // If Root Dir, exit early
        if depth == 0 {
            return;
        }

        world.resource_scope(|world, mut ui_state: Mut<UiState>| {
            let Some(mut row_ui_state) = world.get_mut::<FileSystemUiState>(*row_entity) else {
                return;
            };
            let Some((request_type, request_handle)) = row_ui_state.modal_request.clone() else {
                return;
            };
            let Some(response) = ui_state.text_input_modal.take_response(request_handle) else {
                return;
            };
            row_ui_state.modal_request = None;
            let Some(response_string) = response else {
                return;
            };
            match request_type {
                ModalRequestType::NewFile(directory_entity_opt) => {
                    Self::on_modal_response_new_file(world, directory_entity_opt, response_string);
                }
                ModalRequestType::Delete(row_entity) => {
                    Self::on_modal_response_delete(world, &row_entity);
                }
                ModalRequestType::Rename => {
                    Self::on_modal_response_rename(world, row_entity, response_string);
                }
            }
        });
    }

    pub fn on_modal_response_new_file(
        world: &mut World,
        directory_entity: Option<Entity>,
        new_name: String,
    ) {
        let mut system_state: SystemState<ResMut<ActionStack>> = SystemState::new(world);
        let mut action_stack = system_state.get_mut(world);
        action_stack.buffer_action(Action::NewFile(directory_entity, new_name.clone()));
    }

    pub fn on_modal_response_delete(world: &mut World, row_entity: &Entity) {
        let mut system_state: SystemState<ResMut<ActionStack>> = SystemState::new(world);
        let mut action_stack = system_state.get_mut(world);
        action_stack.buffer_action(Action::DeleteFile(*row_entity));
    }

    pub fn on_modal_response_rename(world: &mut World, row_entity: &Entity, new_name: String) {
        let mut system_state: SystemState<ResMut<ActionStack>> = SystemState::new(world);
        let mut action_stack = system_state.get_mut(world);
        action_stack.buffer_action(Action::RenameFile(*row_entity, new_name.clone()));
    }
}

// fn context_menu(ui: &mut Ui) {
//     // shortcuts
//     let rename_shortcut =
//         egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::R);
//     let delete_shortcut =
//         egui::KeyboardShortcut::new(Modifiers::NONE, egui::Key::Delete);
//     let cut_shortcut =
//         egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::X);
//     let copy_shortcut =
//         egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::C);
//     let paste_shortcut =
//         egui::KeyboardShortcut::new(Modifiers::CTRL, egui::Key::V);
//
//     // NOTE: we must check the shortcuts OUTSIDE of the actual "File" menu,
//     // or else they would only be checked if the "File" menu was actually open!
//
//     // Rename Shortcut
//     if ui.input_mut(|i| i.consume_shortcut(&rename_shortcut)) {
//         // execute some logic
//     }
//     // Delete Shortcut
//     if ui.input_mut(|i| i.consume_shortcut(&delete_shortcut)) {
//         // execute some logic
//     }
//     // Cut Shortcut
//     if ui.input_mut(|i| i.consume_shortcut(&cut_shortcut)) {
//         // execute some logic
//     }
//     // Copy Shortcut
//     if ui.input_mut(|i| i.consume_shortcut(&copy_shortcut)) {
//         // execute some logic
//     }
//     // Paste Shortcut
//     if ui.input_mut(|i| i.consume_shortcut(&paste_shortcut)) {
//         // execute some logic
//     }
// }
