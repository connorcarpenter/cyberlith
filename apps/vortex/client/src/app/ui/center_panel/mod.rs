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

pub fn center_panel(context: &egui::Context, world: &mut World) {
    egui::CentralPanel::default()
        .frame(Frame::none().inner_margin(0.0))
        .show(context, |ui| {
            render_tab_bar(ui, world);
            render_tool_bar(ui, world);

            let canvas = world.get_resource::<Canvas>().unwrap();
            if canvas.current_file_type == FileTypeValue::Anim {
                let animation_manager = world.get_resource::<AnimationManager>().unwrap();
                if animation_manager.current_skel_file.is_none() {
                    render_bind_button(ui, world);
                    return;
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
