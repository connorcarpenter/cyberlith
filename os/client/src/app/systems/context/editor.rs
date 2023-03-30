use bevy_app::{App, Plugin};
use bevy_ecs::prelude::*;
use bevy_log::info;
use render_api::Window;
use render_egui::{egui, EguiContext, EguiContexts, EguiUserTextures, RenderEguiPlugin};

use game_client::GameClientTexture;

pub struct ContextPlugin;

impl Plugin for ContextPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(RenderEguiPlugin).add_system(step);
    }
}

pub fn setup(
    mut egui_user_textures: ResMut<EguiUserTextures>,
    game_client_image: Res<GameClientTexture>,
) {
    egui_user_textures.add_image(game_client_image.0.clone());
}

fn step(
    game_client_image: Res<GameClientTexture>,
    mut contexts: EguiContexts,
    window: Res<Window>,
) {
    // This assumes we only have a single window
    let width = window.resolution.physical_width() / 4;
    let height = window.resolution.physical_height() / 4;

    //TODO: uncomment and make work!
    //
    // let game_client_texture_id = contexts.image_id(&game_client_image.0).unwrap();
    //
    // let ctx = contexts.ctx_mut();
    //
    // egui::Window::new("Game").show(ctx, |ui| {
    //     ui.image(game_client_texture_id, [width as f32, height as f32]);
    // });
}
