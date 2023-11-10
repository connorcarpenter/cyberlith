use bevy_ecs::{entity::Entity, system::{Query, Res, SystemState}, world::{Mut, World}};

use render_egui::{egui::{PointerButton, Vec2, Align, Color32, Frame, Layout, Margin, Sense, Ui}, egui};

use vortex_proto::components::{FileExtension, PaletteColor};

use crate::app::resources::{
    action::icon::IconAction,
    input::IconInputManager,
    tab_manager::TabManager,
    toolbar::Toolbar,
    icon_manager::IconManager,
    file_manager::FileManager,
    palette_manager::PaletteManager,
    shape_data::CanvasShape,
};

pub struct IconToolbar;

impl IconToolbar {
    pub(crate) fn render(ui: &mut Ui, world: &mut World, current_file_entity: &Entity) {
        let icon_manager = world.get_resource::<IconManager>().unwrap();
        let is_framing = icon_manager.is_framing();
        if is_framing {
            egui::SidePanel::right("right_panel")
                .frame(Frame::side_top_panel(ui.style()).inner_margin(Margin {
                    left: 3.0,
                    right: 1.0,
                    top: 2.0,
                    bottom: 2.0,
                }))
                .resizable(false)
                .default_width(26.0)
                .show_inside(ui, |ui| {
                    ui.style_mut().override_text_style = Some(egui::TextStyle::Heading);
                    ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {

                        //
                        Self::framing_render(ui, world);
                        //

                        ui.allocate_space(ui.available_size());
                    });
                    ui.allocate_space(ui.available_size());
                });
        } else {
            world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                Self::posing_render_sidebar(ui, world, &mut icon_manager, current_file_entity);
            });
        }
    }

    fn framing_render(ui: &mut Ui, world: &mut World) {
        button_toggle_play_pause(ui, world);

        // new frame
        if Toolbar::button(ui, "➕", "New frame", true).clicked() {
            world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                IconInputManager::handle_insert_frame(world, &mut icon_manager);
            });
        }

        // delete frame
        if Toolbar::button(ui, "🗑", "Delete frame", true).clicked() {
            world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                IconInputManager::handle_delete_frame(world, &mut icon_manager);
            });
        }

        // move frame left / right
        let current_file_entity = *world
            .get_resource::<TabManager>()
            .unwrap()
            .current_tab_entity()
            .unwrap();
        let icon_manager = world.get_resource::<IconManager>().unwrap();
        let current_frame_index = icon_manager.current_frame_index();
        let frame_count = icon_manager
            .get_frame_count(&current_file_entity)
            .unwrap_or_default();

        {
            // move frame left
            let enabled = current_frame_index > 0;
            let response = Toolbar::button(ui, "⬅", "Move frame left", enabled);
            if enabled && response.clicked() {
                world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_icon_action(
                            world,
                            &mut icon_manager,
                            IconAction::MoveFrame(
                                current_file_entity,
                                current_frame_index,
                                current_frame_index - 1,
                            ),
                        );
                    });
                });
            }
        }

        {
            // move frame right
            let enabled = frame_count > 0 && current_frame_index < frame_count - 1;
            let response = Toolbar::button(ui, "➡", "Move frame right", enabled);
            if enabled && response.clicked() {
                world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                    world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                        tab_manager.current_tab_execute_icon_action(
                            world,
                            &mut icon_manager,
                            IconAction::MoveFrame(
                                current_file_entity,
                                current_frame_index,
                                current_frame_index + 1,
                            ),
                        );
                    });
                });
            }
        }
    }

    // TODO: put this into posing_render_sidebar
    fn posing_render_old(ui: &mut Ui, world: &mut World) {
        // back to framing (up arrow for icon)
        if Toolbar::button(ui, "⬆", "Back to framing", true).clicked() {
            let mut icon_manager = world.get_resource_mut::<IconManager>().unwrap();
            icon_manager.set_framing();
        }

        // insert vertex
        let _response = Toolbar::button(ui, "🔼", "Insert vertex", true);

        // delete selected
        let _response = Toolbar::button(ui, "🗑", "Delete selected shape", true);
    }

    fn posing_render_sidebar(
        ui: &mut Ui,
        world: &mut World,
        icon_manager: &mut IconManager,
        current_file_entity: &Entity,
    ) -> Option<IconAction> {

        let mut color_index_picked = None;

        let mut system_state: SystemState<(
            Res<FileManager>,
            Res<PaletteManager>,
            Query<&PaletteColor>,
        )> = SystemState::new(world);
        let (file_manager, palette_manager, palette_color_q) =
            system_state.get_mut(world);

        let Some(palette_file_entity) = file_manager.file_get_dependency(
            current_file_entity,
            FileExtension::Palette,
        ) else {
            panic!("Expected palette file dependency");
        };
        let Some(colors) = palette_manager.get_file_colors(&palette_file_entity) else {
            return None;
        };

        egui::SidePanel::right("icon_right_panel")
            .exact_width(8.0*2.0 + 48.0*2.0 + 2.0 + 10.0*2.0)
            .resizable(false)
            .show_inside(ui, |ui| {

                let size = Vec2::new(48.0, 48.0);

                ui.horizontal_top(|ui| {
                    Frame::none().inner_margin(8.0).show(ui, |ui| {
                        ui.spacing_mut().item_spacing = Vec2::new(10.0, 10.0);

                        let color_index = icon_manager.selected_color_index();
                        let color_entity_opt = colors.get(color_index).unwrap();
                        let Some(color_entity) = color_entity_opt else {
                            return;
                        };
                        let Ok(color_component) = palette_color_q.get(*color_entity) else {
                            return;
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
                                    if color_index == icon_manager.selected_color_index() {
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
                if color_index_picked == icon_manager.selected_color_index() {
                    return None;
                }
                icon_manager.set_selected_color_index(color_index_picked);

                let selected_shape = icon_manager.selected_shape();
                if selected_shape.is_some() {
                    let Some((face_entity, CanvasShape::Face)) = selected_shape else {
                        panic!("expected face entity");
                    };
                    return Some(IconAction::EditColor(
                        face_entity,
                        Some(palette_color_entity),
                    ));
                }
            }
            _ => {}
        }

        return None;
    }
}

fn button_toggle_play_pause(ui: &mut Ui, world: &mut World) {
    // play / pause button
    let mut icon_manager = world.get_resource_mut::<IconManager>().unwrap();
    if icon_manager.preview_is_playing() {
        if Toolbar::button(ui, "⏸", "Pause", true).clicked() {
            icon_manager.preview_pause();
        }
    } else {
        if Toolbar::button(ui, "▶", "Play", true).clicked() {
            icon_manager.preview_play();
        }
    }
}
