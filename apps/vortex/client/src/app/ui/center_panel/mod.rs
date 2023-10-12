mod canvas;
use canvas::render_canvas;

use bevy_ecs::world::{Mut, World};

use render_egui::{egui, egui::Frame};

use vortex_proto::components::FileExtension;

use crate::app::{
    resources::{
        input_manager::InputManager,
        animation_manager::AnimationManager, file_manager::FileManager,
        palette_manager::PaletteManager, skin_manager::SkinManager, tab_manager::render_tab_bar,
        tab_manager::TabManager,
    },
    ui::{
        render_tool_bar,
        widgets::{
            render_bind_button, render_frame_inspect_bar, render_naming_bar, NamingBarState,
        },
    },
};

pub fn center_panel(context: &egui::Context, world: &mut World) {
    egui::CentralPanel::default()
        .frame(Frame::none().inner_margin(0.0))
        .show(context, |ui| {
            render_tab_bar(ui, world);

            let mut render_frame_inspect = false;
            let tab_manager = world.get_resource::<TabManager>().unwrap();
            if let Some(current_file_entity) = tab_manager.current_tab_entity() {
                let current_file_entity = *current_file_entity;
                let file_manager = world.get_resource::<FileManager>().unwrap();
                let current_file_type = file_manager.get_file_type(&current_file_entity);

                match current_file_type {
                    FileExtension::Skel | FileExtension::Mesh => {
                        render_tool_bar(ui, world, current_file_type);
                    }
                    FileExtension::Anim => {
                        let file_manager = world.get_resource::<FileManager>().unwrap();
                        if !file_manager.file_has_dependency_with_extension(
                            &current_file_entity,
                            FileExtension::Skel,
                        ) {
                            render_bind_button(
                                ui,
                                world,
                                &current_file_entity,
                                FileExtension::Skel,
                            );
                            return;
                        }

                        render_tool_bar(ui, world, current_file_type);

                        let animation_manager = world.get_resource::<AnimationManager>().unwrap();
                        if animation_manager.is_framing() {
                            render_frame_inspect = true;
                        }
                    }
                    FileExtension::Palette => {
                        PaletteManager::render(ui, world, &current_file_entity);
                        return;
                    }
                    FileExtension::Skin => {
                        let file_manager = world.get_resource::<FileManager>().unwrap();

                        // Palette Dependency
                        if !file_manager.file_has_dependency_with_extension(
                            &current_file_entity,
                            FileExtension::Palette,
                        ) {
                            render_bind_button(
                                ui,
                                world,
                                &current_file_entity,
                                FileExtension::Palette,
                            );
                            return;
                        }

                        // Mesh Dependency
                        if !file_manager.file_has_dependency_with_extension(
                            &current_file_entity,
                            FileExtension::Mesh,
                        ) {
                            render_bind_button(
                                ui,
                                world,
                                &current_file_entity,
                                FileExtension::Mesh,
                            );
                            return;
                        }

                        // Toolbar
                        let mut some_action = None;
                        world.resource_scope(|world, mut skin_manager: Mut<SkinManager>| {
                            some_action = skin_manager.render_sidebar(ui, world, &current_file_entity);
                        });
                        if let Some(skin_action) = some_action {
                            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                                world.resource_scope(|world, mut input_manager: Mut<InputManager>| {
                                    tab_manager.current_tab_execute_skin_action(
                                        world,
                                        &mut input_manager,
                                        skin_action,
                                    );
                                });
                            });
                        }
                    }
                    _ => {}
                }
            }

            let naming_bar = world.get_resource::<NamingBarState>().unwrap();
            if naming_bar.visible {
                egui::CentralPanel::default() // canvas area
                    .frame(Frame::central_panel(ui.style()).inner_margin(0.0))
                    .show_inside(ui, |ui| {
                        render_naming_bar(ui, world);
                        render_canvas(ui, world);
                    });
            } else {
                render_canvas(ui, world);
            }

            if render_frame_inspect {
                render_frame_inspect_bar(ui, world);
            }
        });
}
