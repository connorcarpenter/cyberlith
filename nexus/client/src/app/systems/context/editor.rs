use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_render::{egui, EguiContext, EguiUserTextures, Window};

use game_client::GameClientImage;

pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(step);
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
    mut context: ResMut<EguiContext>,
    window: Res<Window>,
) {
    // This assumes we only have a single window
    let width = window.resolution.physical_width() / 4.0;
    let height = window.resolution.physical_height() / 4.0;

    let game_client_texture_id = context.image_id(&game_client_image.0).unwrap();

    let ctx = context.ctx_mut();

    egui::Window::new("Game").show(ctx, |ui| {
        ui.image(game_client_texture_id, [width as f32, height as f32]);
    });
}
