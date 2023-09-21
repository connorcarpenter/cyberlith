use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res, ResMut, Resource, SystemState},
    world::World,
};

use render_egui::{
    egui,
    egui::{Align, Button, Frame, Layout, TextEdit, Ui},
};

use vortex_proto::components::{FileExtension, ShapeName};

use crate::app::{
    resources::{
        animation_manager::AnimationManager, canvas::Canvas, edge_manager::EdgeManager,
        face_manager::FaceManager, file_manager::FileManager, input_manager::InputManager,
        shape_data::CanvasShape, shape_manager::ShapeManager, tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
    ui::UiState,
};

#[derive(Resource)]
pub struct NamingBarState {
    pub(crate) visible: bool,
    prev_text: String,
    text: String,
    pub(crate) selected_shape_opt: Option<(Entity, CanvasShape)>,
}

impl Default for NamingBarState {
    fn default() -> Self {
        Self {
            visible: false,
            prev_text: "".to_string(),
            text: "".to_string(),
            selected_shape_opt: None,
        }
    }
}

pub fn render_naming_bar(ui: &mut Ui, world: &mut World) {
    egui::TopBottomPanel::top("naming_bar")
        .frame(Frame::central_panel(ui.style()).inner_margin(2.0))
        .show_inside(ui, |ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let input_manager = world.get_resource::<InputManager>().unwrap();
                let selected_shape_2d = input_manager.selected_shape_2d();

                let state = world.get_resource::<NamingBarState>().unwrap();
                if state.selected_shape_opt != selected_shape_2d {
                    let mut system_state: SystemState<(
                        ResMut<NamingBarState>,
                        Res<VertexManager>,
                        Res<EdgeManager>,
                        Res<FaceManager>,
                        Query<&ShapeName>,
                    )> = SystemState::new(world);
                    let (mut state, vertex_manager, edge_manager, face_manager, shape_name_q) =
                        system_state.get_mut(world);

                    let official_name = if let Some((shape_2d_entity, shape)) = selected_shape_2d {
                        let shape_3d_entity = ShapeManager::shape_entity_2d_to_3d(
                            &vertex_manager,
                            &edge_manager,
                            &face_manager,
                            &shape_2d_entity,
                            shape,
                        )
                        .unwrap();

                        if let Ok(shape_name) = shape_name_q.get(shape_3d_entity) {
                            (*shape_name.value).clone()
                        } else {
                            "".to_string()
                        }
                    } else {
                        "".to_string()
                    };
                    state.prev_text = official_name.clone();
                    state.text = official_name;
                    state.selected_shape_opt = selected_shape_2d;

                    system_state.apply(world);
                }

                let state = world.get_resource::<NamingBarState>().unwrap();
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
                    let mut state = world.get_resource_mut::<NamingBarState>().unwrap();
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
                    let (shape_2d_entity, shape) = selected_shape_2d.unwrap();

                    let shape_3d_entity = ShapeManager::shape_entity_2d_to_3d(
                        world.get_resource::<VertexManager>().unwrap(),
                        world.get_resource::<EdgeManager>().unwrap(),
                        world.get_resource::<FaceManager>().unwrap(),
                        &shape_2d_entity,
                        shape,
                    )
                    .unwrap();

                    let mut system_state: SystemState<(
                        Commands,
                        ResMut<NamingBarState>,
                        Query<Option<&mut ShapeName>>,
                    )> = SystemState::new(world);
                    let (mut commands, mut state, mut shape_q) = system_state.get_mut(world);

                    state.prev_text = state.text.clone();
                    let mut shape_name_opt = shape_q.get_mut(shape_3d_entity).unwrap();
                    if let Some(shape_name) = shape_name_opt.as_mut() {
                        *shape_name.value = state.text.clone();
                    } else {
                        commands
                            .entity(shape_3d_entity)
                            .insert(ShapeName::new(state.text.clone()));
                    }

                    system_state.apply(world);
                }

                let mut state = world.get_resource_mut::<NamingBarState>().unwrap();
                let text_edit = TextEdit::singleline(&mut state.text);
                let response = ui.add_enabled(selected_shape_2d.is_some(), text_edit);
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

                ui.label("name: ");
            });
        });
}

pub fn naming_bar_visibility_toggle(world: &mut World) {
    // is skeleton toolbar open?

    // get current file extension
    let Some(current_file_entity) = world.get_resource::<TabManager>().unwrap().current_tab_entity() else {
        return;
    };
    let current_file_type = world
        .get_resource::<FileManager>()
        .unwrap()
        .get_file_type(&current_file_entity);
    if current_file_type != FileExtension::Skel {
        return;
    }

    // is vertex/edge selected?
    let input_manager = world.get_resource::<InputManager>().unwrap();
    let selected_shape_2d = input_manager.selected_shape_2d();
    if selected_shape_2d.is_none() {
        return;
    }

    // actually toggle
    let mut ui_state = world.get_resource_mut::<NamingBarState>().unwrap();
    let old_visible = ui_state.visible;
    ui_state.visible = !old_visible;

    let mut ui_state = world.get_resource_mut::<UiState>().unwrap();
    ui_state.resized_window = true;

    // set focus to canvas
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

    canvas.set_focused_timed(
        &mut input_manager,
        &mut vertex_manager,
        &mut edge_manager,
        &mut animation_manager,
    );

    system_state.apply(world);
}
