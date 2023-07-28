use bevy_ecs::{
    prelude::Commands,
    system::{Query, ResMut},
};

use naia_bevy_client::Client;

use render_api::components::{Camera, Transform};

use crate::app::resources::{canvas_manager::CanvasManager, action_stack::ActionStack};

pub fn step(
    mut commands: Commands,
    client: Client,
    mut canvas_manager: ResMut<CanvasManager>,
    mut action_stack: ResMut<ActionStack>,
    mut camera_q: Query<(&mut Camera, &mut Transform)>,
) {
    canvas_manager.update_visibility(&mut camera_q);
    canvas_manager.update_3d_camera(&mut camera_q);
    canvas_manager.poll_buffered_actions(&mut commands, &client, &mut action_stack);
}
