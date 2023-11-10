mod canvas;

use canvas::render_canvas;

use bevy_ecs::world::{Mut, World};

use render_egui::{egui, egui::Frame};

use vortex_proto::components::FileExtension;

use crate::app::{
    resources::{
        animation_manager::AnimationManager, file_manager::FileManager, input::InputManager,
        model_manager::ModelManager, palette_manager::PaletteManager, skin_manager::SkinManager,
        tab_manager::render_tab_bar, tab_manager::TabManager,
    },
    ui::{
        render_tool_bar,
        widgets::{
            render_bind_button, render_frame_inspect_bar, render_naming_bar, render_simple_bind,
            NamingBarState,
        },
    },
};
use crate::app::resources::toolbar::{IconToolbar, SkinToolbar};

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
                        render_tool_bar(ui, world, &current_file_entity, current_file_type);
                    }
                    FileExtension::Anim => {
                        if !render_simple_bind(world, ui, &current_file_entity, FileExtension::Skel)
                        {
                            return;
                        }

                        render_tool_bar(ui, world, &current_file_entity, current_file_type);

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
                        // Palette Dependency
                        if !render_simple_bind(
                            world,
                            ui,
                            &current_file_entity,
                            FileExtension::Palette,
                        ) {
                            return;
                        }

                        // Mesh Dependency
                        if !render_simple_bind(world, ui, &current_file_entity, FileExtension::Mesh)
                        {
                            return;
                        }

                        SkinToolbar::render(world, ui, &current_file_entity);
                    }
                    FileExtension::Model => {
                        if !render_simple_bind(world, ui, &current_file_entity, FileExtension::Skel)
                        {
                            return;
                        }

                        if world
                            .get_resource::<ModelManager>()
                            .unwrap()
                            .transform_is_binding()
                        {
                            if let Some((dependency_file_ext, dependency_file_entity)) =
                                render_bind_button(
                                    ui,
                                    world,
                                    &[FileExtension::Skin, FileExtension::Scene],
                                )
                            {
                                let edge_2d_entity = world
                                    .get_resource_mut::<ModelManager>()
                                    .unwrap()
                                    .take_binding_result()
                                    .unwrap();
                                ModelManager::process_render_bind_button_result(
                                    world,
                                    &current_file_entity,
                                    &dependency_file_ext,
                                    &dependency_file_entity,
                                    Some(&edge_2d_entity),
                                );
                            }
                            return;
                        }

                        render_tool_bar(ui, world, &current_file_entity, current_file_type);
                    }
                    FileExtension::Scene => {
                        if world
                            .get_resource::<ModelManager>()
                            .unwrap()
                            .transform_is_binding()
                        {
                            if let Some((dependency_file_ext, dependency_file_entity)) =
                                render_bind_button(
                                    ui,
                                    world,
                                    &[FileExtension::Skin, FileExtension::Scene],
                                )
                            {
                                if world
                                    .get_resource_mut::<ModelManager>()
                                    .unwrap()
                                    .take_binding_result()
                                    .is_some()
                                {
                                    panic!("Should not get an edge entity in this case");
                                }
                                ModelManager::process_render_bind_button_result(
                                    world,
                                    &current_file_entity,
                                    &dependency_file_ext,
                                    &dependency_file_entity,
                                    None,
                                );
                            }
                            return;
                        }

                        render_tool_bar(ui, world, &current_file_entity, current_file_type);
                    }
                    FileExtension::Icon => {
                        // Palette Dependency
                        if !render_simple_bind(
                            world,
                            ui,
                            &current_file_entity,
                            FileExtension::Palette,
                        ) {
                            return;
                        }

                        // Toolbar
                        IconToolbar::render(ui, world, &current_file_entity);
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
