mod canvas;

use bevy_ecs::world::World;

use canvas::render_canvas;
use render_egui::{egui, egui::Frame};
use vortex_proto::components::FileTypeValue;

use crate::app::{
    resources::{tab_manager::render_tab_bar, canvas::Canvas, animation_manager::AnimationManager},
    ui::{
        render_tool_bar,
        widgets::{render_bind_button, render_naming_bar, NamingBarState},
    },
};
use crate::app::resources::file_manager::FileManager;
use crate::app::resources::tab_manager::TabManager;
use crate::app::ui::widgets::render_bound;

pub fn center_panel(context: &egui::Context, world: &mut World) {
    egui::CentralPanel::default()
        .frame(Frame::none().inner_margin(0.0))
        .show(context, |ui| {
            render_tab_bar(ui, world);
            render_tool_bar(ui, world);

            let tab_manager = world.get_resource::<TabManager>().unwrap();
            if let Some(current_file_entity) = tab_manager.current_tab_entity() {
                let current_file_entity = *current_file_entity;
                // let mut entities = world.query::<(Entity, &Order, &Label)>()
                //     .iter(&world)
                //     .collect::<Vec<_>>();
                let canvas = world.get_resource::<Canvas>().unwrap();
                if canvas.current_file_type_equals(FileTypeValue::Anim) {
                    let file_manager = world.get_resource::<FileManager>().unwrap();
                    if !file_manager.file_has_dependency(&current_file_entity, FileTypeValue::Skel) {
                        render_bind_button(ui, world, current_file_entity);
                        return;
                    } else {
                        render_bound(ui, world, current_file_entity);
                        return;
                    }
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
        });
}
