use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query, Res, ResMut, Resource, SystemState},
    world::{Mut, World},
};

use render_egui::{
    egui,
    egui::{Align, Button, Frame, Layout, TextEdit, Ui},
};
use vortex_proto::components::ShapeName;

use crate::app::{
    resources::{
        shape_data::CanvasShape,
        canvas::Canvas,
        shape_manager::ShapeManager,
        toolbar::{Toolbar, ToolbarKind},
    },
    ui::UiState,
};
use crate::app::resources::edge_manager::EdgeManager;
use crate::app::resources::face_manager::FaceManager;
use crate::app::resources::vertex_manager::VertexManager;

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
                let shape_manager = world.get_resource::<ShapeManager>().unwrap();
                let selected_shape_2d = shape_manager.selected_shape_2d();

                let state = world.get_resource::<NamingBarState>().unwrap();
                if state.selected_shape_opt != selected_shape_2d {
                    let mut system_state: SystemState<(
                        ResMut<NamingBarState>,
                        Res<VertexManager>,
                        Res<EdgeManager>,
                        Res<FaceManager>,
                        Query<&ShapeName>,
                    )> = SystemState::new(world);
                    let (mut state, vertex_manager, edge_manager, face_manager, shape_name_q) = system_state.get_mut(world);

                    let official_name = if let Some((shape_2d_entity, shape)) = selected_shape_2d {
                        let shape_3d_entity = ShapeManager::shape_entity_2d_to_3d(&vertex_manager, &edge_manager, &face_manager,&shape_2d_entity, shape)
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
                        shape
                    ).unwrap();

                    let mut system_state: SystemState<(
                        Commands,
                        ResMut<NamingBarState>,
                        Query<&mut ShapeName>,
                    )> = SystemState::new(world);
                    let (mut commands, mut state, mut shape_name_q) = system_state.get_mut(world);

                    state.prev_text = state.text.clone();
                    if let Ok(mut shape_name) = shape_name_q.get_mut(shape_3d_entity) {
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
                        ResMut<ShapeManager>,
                        ResMut<VertexManager>,
                        ResMut<EdgeManager>,
                    )> = SystemState::new(world);
                    let (mut canvas, mut shape_manager, mut vertex_manager, mut edge_manager) = system_state.get_mut(world);

                    canvas.set_focus(
                        &mut shape_manager,
                        &mut vertex_manager,
                        &mut edge_manager,
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
    let toolbar = world.get_resource::<Toolbar>().unwrap();
    let toolbar_kind = toolbar.kind();
    if toolbar_kind != Some(ToolbarKind::Skeleton) {
        return;
    }

    // is vertex/edge selected?
    let shape_manager = world.get_resource::<ShapeManager>().unwrap();
    let selected_shape_2d = shape_manager.selected_shape_2d();
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
        ResMut<ShapeManager>,
        ResMut<VertexManager>,
        ResMut<EdgeManager>,
    )> = SystemState::new(world);
    let (
        mut canvas,
        mut shape_manager,
        mut vertex_manager,
        mut edge_manager,
    ) = system_state.get_mut(world);

    canvas.set_focused_timed(&mut shape_manager, &mut vertex_manager, &mut edge_manager);

    system_state.apply(world);
}
