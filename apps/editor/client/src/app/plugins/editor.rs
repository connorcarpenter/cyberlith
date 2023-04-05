use bevy_app::{App, Plugin};
use bevy_ecs::{
    component::Component,
    query::With,
    schedule::IntoSystemConfigs,
    system::{Commands, Local, Query, Res, ResMut, Resource},
};

use naia_bevy_client::{
    ClientConfig as NaiaClientConfig, Plugin as NaiaClientPlugin, ReceiveEvents,
};

use math::Vec3;
use render_api::{
    base::{Camera, Color, PbrMaterial, TriMesh, Viewport},
    shape, AmbientLight, Assets, CameraComponent, ClearOperation, DirectionalLight, PointLight,
    RenderObjectBundle, RenderTarget, Transform, Window,
};
use render_egui::{egui, EguiContext, GUI};

use editor_proto::protocol;

use crate::app::network;

#[derive(Component)]
pub struct CubeMarker;

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app
            // Add Naia Client Plugin
            // .add_plugin(NaiaClientPlugin::new(
            //     NaiaClientConfig::default(),
            //     protocol(),
            // ))
            // Startup Systems
            // .add_startup_system(network::init)
            .init_resource::<OccupiedScreenSpace>()
            .add_startup_system(setup)
            .add_system(ui_example_system)
            // Receive Client Events
            // .add_systems(
            //     (
            //         network::connect_events,
            //         network::disconnect_events,
            //         network::reject_events,
            //         network::error_events,
            //     )
            //         .chain()
            //         .in_set(ReceiveEvents),
            // )
            // .add_system(step);
        ;
    }
}

#[derive(Default, Resource)]
struct OccupiedScreenSpace {
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

fn setup(mut commands: Commands, window: Res<Window>) {
    let width = window.resolution.physical_width();
    let height = window.resolution.physical_height();

    // camera
    commands.spawn(CameraComponent::new(
        Camera::new_orthographic(
            Viewport::new_at_origin(width, height),
            Vec3::new(50.0, 50.0, 50.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            50.0,
            0.0,
            1000.0,
        ),
        // render before the "main pass" camera
        0,
        ClearOperation::default(),
        RenderTarget::Screen,
    ));
}

fn ui_example_system(
    context: Res<EguiContext>,
    mut occupied_screen_space: ResMut<OccupiedScreenSpace>,
) {
    occupied_screen_space.left = egui::SidePanel::left("left_panel")
        .resizable(true)
        .show(context.inner(), |ui| {
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    occupied_screen_space.right = egui::SidePanel::right("right_panel")
        .resizable(true)
        .show(context.inner(), |ui| {
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .width();
    occupied_screen_space.top = egui::TopBottomPanel::top("top_panel")
        .resizable(true)
        .show(context.inner(), |ui| {
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
    occupied_screen_space.bottom = egui::TopBottomPanel::bottom("bottom_panel")
        .resizable(true)
        .show(context.inner(), |ui| {
            ui.allocate_rect(ui.available_rect_before_wrap(), egui::Sense::hover());
        })
        .response
        .rect
        .height();
}
