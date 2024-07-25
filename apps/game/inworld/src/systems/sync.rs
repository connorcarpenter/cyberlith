use bevy_ecs::prelude::{Query, With};

use game_engine::{world::{WorldClient, components::Position}, render::components::Transform};

use crate::components::{Confirmed, Interp, Predicted};

pub fn sync_clientside_sprites(
    client: WorldClient,
    mut query: Query<(&Position, &mut Interp, &mut Transform), With<Predicted>>,
) {
    for (position, mut interp, mut transform) in query.iter_mut() {
        if position.x != interp.next_x || position.y != interp.next_y {
            interp.next_position(position.x, position.y);
        }

        let interp_amount = client.client_interpolation().unwrap();
        interp.interpolate(interp_amount);
        transform.translation.x = interp.interp_x;
        transform.translation.y = interp.interp_y;
    }
}

pub fn sync_serverside_sprites(
    client: WorldClient,
    mut query: Query<(&Position, &mut Interp, &mut Transform), With<Confirmed>>,
) {
    for (position, mut interp, mut transform) in query.iter_mut() {
        if position.x != interp.next_x || position.y != interp.next_y {
            interp.next_position(position.x, position.y);
        }

        let interp_amount = client.server_interpolation().unwrap();
        interp.interpolate(interp_amount);
        transform.translation.x = interp.interp_x;
        transform.translation.y = interp.interp_y;
    }
}