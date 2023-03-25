use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin, EguiUserTextures};
use bevy_window::Window;

use cybl_game_client::GameClientImage;

pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin).add_system(step);
    }
}

pub fn setup(
    mut egui_user_textures: ResMut<EguiUserTextures>,
    game_client_image: Res<GameClientImage>,
) {
    egui_user_textures.add_image(game_client_image.0.clone());
}

fn step(
    game_client_image: Res<GameClientImage>,
    mut contexts: EguiContexts,
    windows: Query<&Window>,
) {
    // This assumes we only have a single window
    let window = windows.single();
    let width = window.resolution.physical_width() / 4;
    let height = window.resolution.physical_height() / 4;

    let game_client_texture_id = contexts.image_id(&game_client_image.0).unwrap();

    let ctx = contexts.ctx_mut();

    egui::Window::new("Game").show(ctx, |ui| {
        ui.image(game_client_texture_id, [width as f32, height as f32]);
    });
}
