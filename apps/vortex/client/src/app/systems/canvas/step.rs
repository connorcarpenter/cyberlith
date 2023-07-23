use bevy_ecs::{
    entity::Entity,
    prelude::Commands,
    system::{Query, ResMut},
};

use naia_bevy_client::Client;

use render_api::components::{Camera, Transform};

use crate::app::{
    components::{Edge2d, Edge3d},
    resources::canvas_manager::CanvasManager,
};

pub fn step(
    mut commands: Commands,
    client: Client,
    mut canvas_manager: ResMut<CanvasManager>,
    mut camera_q: Query<(&mut Camera, &mut Transform)>,
    edge_3d_q: Query<(Entity, &Edge3d)>,
    edge_2d_q: Query<(Entity, &Edge2d)>,
) {
    canvas_manager.update_visibility(&mut camera_q);
    canvas_manager.update_3d_camera(&mut camera_q);
    canvas_manager.poll_buffered_actions(&mut commands, &client, &edge_3d_q, &edge_2d_q);
}
