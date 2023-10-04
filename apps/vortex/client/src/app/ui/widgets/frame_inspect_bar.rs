use bevy_ecs::{
    entity::Entity,
    system::{Query, ResMut, Resource, SystemState},
    world::World,
};

use render_egui::{
    egui,
    egui::{Align, Button, Frame, Layout, TextEdit, Ui},
};

use vortex_proto::components::AnimFrame;

use crate::app::resources::{
    animation_manager::AnimationManager, canvas::Canvas, edge_manager::EdgeManager,
    input_manager::InputManager,
    tab_manager::TabManager,
    vertex_manager::VertexManager,
};

#[derive(Resource)]
pub struct FrameInspectBarState {
    prev_text: String,
    text: String,
    selected_frame_opt: Option<Entity>,
}

impl Default for FrameInspectBarState {
    fn default() -> Self {
        Self {
            prev_text: "".to_string(),
            text: "".to_string(),
            selected_frame_opt: None,
        }
    }
}

pub fn render_frame_inspect_bar(ui: &mut Ui, world: &mut World) {
    egui::TopBottomPanel::bottom("frame_inspect_bar")
        .frame(Frame::central_panel(ui.style()).inner_margin(2.0))
        .show_inside(ui, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let tab_manager = world.get_resource::<TabManager>().unwrap();
                let current_file_entity = *tab_manager.current_tab_entity().unwrap();
                let animation_manager = world.get_resource::<AnimationManager>().unwrap();
                let selected_frame_entity = animation_manager.current_frame_entity(&current_file_entity);

                let state = world.get_resource::<FrameInspectBarState>().unwrap();
                if state.selected_frame_opt != selected_frame_entity {
                    let mut system_state: SystemState<(
                        ResMut<FrameInspectBarState>,
                        Query<&AnimFrame>,
                    )> = SystemState::new(world);
                    let (mut state, frame_q) =
                        system_state.get_mut(world);

                    let official_name = if let Some(frame_entity) = selected_frame_entity {
                        if let Ok(frame) = frame_q.get(frame_entity) {
                            frame.transition.get_duration_ms().to_string()
                        } else {
                            "".to_string()
                        }
                    } else {
                        "".to_string()
                    };
                    state.prev_text = official_name.clone();
                    state.text = official_name;
                    state.selected_frame_opt = selected_frame_entity;

                    system_state.apply(world);
                }

                let state = world.get_resource::<FrameInspectBarState>().unwrap();
                let has_changed = state.prev_text != state.text;

                // cancel button
                if ui
                    .add_enabled(
                        has_changed,
                        Button::new("✖").min_size(egui::Vec2::splat(18.0)),
                    )
                    .on_hover_text("Cancel")
                    .clicked()
                {
                    let mut state = world.get_resource_mut::<FrameInspectBarState>().unwrap();
                    state.text = state.prev_text.clone();
                }

                // accept button
                if ui
                    .add_enabled(
                        has_changed,
                        Button::new("✔").min_size(egui::Vec2::splat(18.0)),
                    )
                    .on_hover_text("Accept")
                    .clicked()
                {
                    let frame_entity = selected_frame_entity.unwrap();

                    let mut system_state: SystemState<(
                        ResMut<FrameInspectBarState>,
                        Query<&mut AnimFrame>,
                    )> = SystemState::new(world);
                    let (mut state, mut frame_q) = system_state.get_mut(world);

                    state.prev_text = state.text.clone();
                    let mut frame = frame_q.get_mut(frame_entity).unwrap();
                    if let Ok(new_duration_ms) = state.text.parse::<u16>() {
                        frame.transition.set_duration_ms(new_duration_ms);
                    }

                    system_state.apply(world);
                }

                ui.label("milliseconds");

                let mut state = world.get_resource_mut::<FrameInspectBarState>().unwrap();
                let text_edit = TextEdit::singleline(&mut state.text);
                let response = ui.add_enabled(selected_frame_entity.is_some(), text_edit);
                if response.has_focus() {
                    let mut system_state: SystemState<(
                        ResMut<Canvas>,
                        ResMut<InputManager>,
                        ResMut<VertexManager>,
                        ResMut<EdgeManager>,
                        ResMut<AnimationManager>,
                    )> = SystemState::new(world);
                    let (
                        mut canvas,
                        mut input_manager,
                        mut vertex_manager,
                        mut edge_manager,
                        mut animation_manager,
                    ) = system_state.get_mut(world);

                    canvas.set_focus(
                        &mut input_manager,
                        &mut vertex_manager,
                        &mut edge_manager,
                        &mut animation_manager,
                        false,
                    );

                    system_state.apply(world);
                }

                ui.label("frame duration: ");
            });
        });
}

// pub fn naming_bar_visibility_toggle(world: &mut World, input_manager: &mut InputManager) {
//     // is skeleton toolbar open?
//
//     // get current file extension
//     let Some(current_file_entity) = world.get_resource::<TabManager>().unwrap().current_tab_entity() else {
//         return;
//     };
//     let current_file_type = world
//         .get_resource::<FileManager>()
//         .unwrap()
//         .get_file_type(&current_file_entity);
//     if current_file_type != FileExtension::Skel {
//         return;
//     }
//
//     // is vertex/edge selected?
//     let selected_shape_2d = input_manager.selected_shape_2d();
//     if selected_shape_2d.is_none() {
//         return;
//     }
//
//     // actually toggle
//     let mut ui_state = world.get_resource_mut::<NamingBarState>().unwrap();
//     let old_visible = ui_state.visible;
//     ui_state.visible = !old_visible;
//
//     let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
//     ui_state.resized_window = true;
//
//     // set focus to canvas
//     let mut system_state: SystemState<(
//         ResMut<Canvas>,
//         ResMut<VertexManager>,
//         ResMut<EdgeManager>,
//         ResMut<AnimationManager>,
//     )> = SystemState::new(world);
//     let (mut canvas, mut vertex_manager, mut edge_manager, mut animation_manager) =
//         system_state.get_mut(world);
//
//     canvas.set_focused_timed(
//         input_manager,
//         &mut vertex_manager,
//         &mut edge_manager,
//         &mut animation_manager,
//     );
//
//     system_state.apply(world);
// }
