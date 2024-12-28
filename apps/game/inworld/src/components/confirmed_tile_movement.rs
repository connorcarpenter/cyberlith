use bevy_ecs::prelude::Component;

use game_engine::{
    logging::{info, warn},
    math::Vec2,
};

use game_app_network::{
    naia::{sequence_greater_than, Tick},
    world::{
        components::{
            MoveBuffer, NetworkedMoveBuffer, NetworkedTileTarget, PhysicsController, TileMovement,
            TileMovementType,
        },
        constants::TILE_SIZE,
        types::Direction,
    },
};

use crate::{
    components::{client_tile_movement::ClientTileMovement, AnimationState, RenderPosition},
    resources::TickTracker,
    systems::world_events::process_tick,
};

#[derive(Component, Clone)]
pub struct ConfirmedTileMovement {
    tile_movement: TileMovement,
    move_buffer: MoveBuffer,
}

impl ClientTileMovement for ConfirmedTileMovement {
    fn decompose(&mut self) -> (&mut TileMovement, &mut MoveBuffer) {
        (&mut self.tile_movement, &mut self.move_buffer)
    }
}

impl ConfirmedTileMovement {
    pub fn new_stopped(next_tile_position: &NetworkedTileTarget) -> Self {
        Self {
            tile_movement: TileMovement::new_stopped(next_tile_position),
            move_buffer: MoveBuffer::new(),
        }
    }

    pub fn recv_updated_next_tile_position(
        &mut self,
        tick_tracker: &TickTracker,
        update_tick: Tick,
        next_tile_position: &NetworkedTileTarget,
        physics: &mut PhysicsController,
        render_position: &mut RenderPosition,
        animation_state: &mut AnimationState,
    ) {
        let (next_velocity_x, next_velocity_y) = (
            next_tile_position.velocity_x(),
            next_tile_position.velocity_y(),
        );
        info!(
            "Recv NextTilePosition. Tick: {:?}, Next Tile: ({:?}, {:?}), Velocity: ({:?}, {:?})",
            update_tick,
            next_tile_position.x(),
            next_tile_position.y(),
            next_velocity_x,
            next_velocity_y
        );

        physics.set_velocity(next_velocity_x, next_velocity_y, true);

        let (new_current_tile_x, new_current_tile_y, next_move_dir) = {
            if self.tile_movement.is_stopped() {
                // Is Stopped
                let (next_tile_x, next_tile_y) = (next_tile_position.x(), next_tile_position.y());
                let stopped_state = self.tile_movement.as_stopped_mut();
                let (current_tile_x, current_tile_y) = stopped_state.tile_position();
                if current_tile_x == next_tile_x && current_tile_y == next_tile_y {
                    panic!(
                        "Unexpected! Current tile position is the same as the next tile position"
                    );
                } else {
                    let (new_current_tile_x, new_current_tile_y, new_move_dir) = {
                        let dx = (next_tile_x - current_tile_x) as i8;
                        let dy = (next_tile_y - current_tile_y) as i8;

                        if let Some(move_dir) = Direction::from_delta(dx, dy) {
                            (current_tile_x, current_tile_y, move_dir)
                        } else {
                            warn!(
                                "Invalid move direction. Prev: ({:?}, {:?}), Next: ({:?}, {:?}). Pathfinding...",
                                current_tile_x, current_tile_y, next_tile_x, next_tile_y
                            );

                            pathfind_to_tile(
                                current_tile_x,
                                current_tile_y,
                                next_tile_x,
                                next_tile_y,
                            )
                        }
                    };
                    stopped_state.set_tile_position(new_current_tile_x, new_current_tile_y);
                    (new_current_tile_x, new_current_tile_y, new_move_dir)
                }
            } else {
                // Is Moving
                let moving_state = self.tile_movement.as_moving();

                let (
                    current_from_tile_x,
                    current_from_tile_y,
                    current_to_tile_x,
                    current_to_tile_y,
                ) = moving_state.tile_positions();
                let interpolation: f32 = {
                    let current_position = physics.position();
                    let from_position = Vec2::new(
                        current_from_tile_x as f32 * TILE_SIZE,
                        current_from_tile_y as f32 * TILE_SIZE,
                    );
                    let to_position = Vec2::new(
                        current_to_tile_x as f32 * TILE_SIZE,
                        current_to_tile_y as f32 * TILE_SIZE,
                    );
                    let from_dis = current_position.distance(from_position);
                    let to_dis = current_position.distance(to_position);
                    let total_dis = from_dis + to_dis;
                    from_dis / total_dis
                };
                let (next_to_tile_x, next_to_tile_y) =
                    (next_tile_position.x(), next_tile_position.y());
                let (next_from_tile_x, next_from_tile_y, next_move_dir) = {
                    let (next_from_tile_x, next_from_tile_y) = if interpolation < 0.5 {
                        (current_from_tile_x, current_from_tile_y)
                    } else {
                        if current_to_tile_x == next_to_tile_x
                            && current_to_tile_y == next_to_tile_y
                        {
                            // The mover is already moving to the next tile from the correct tile
                            (current_from_tile_x, current_from_tile_y)
                        } else {
                            (current_to_tile_x, current_to_tile_y)
                        }
                    };

                    let next_tile_dx = (next_to_tile_x - next_from_tile_x) as i8;
                    let next_tile_dy = (next_to_tile_y - next_from_tile_y) as i8;
                    if let Some(move_dir) = Direction::from_delta(next_tile_dx, next_tile_dy) {
                        (next_from_tile_x, next_from_tile_y, move_dir)
                    } else {
                        warn!(
                            "Invalid move direction. Prev: ({:?}, {:?}), Next: ({:?}, {:?}). Pathfinding...",
                            next_from_tile_x, next_from_tile_y, next_to_tile_x, next_to_tile_y
                        );

                        pathfind_to_tile(
                            next_from_tile_x,
                            next_from_tile_y,
                            next_to_tile_x,
                            next_to_tile_y,
                        )
                    }
                };

                self.tile_movement
                    .set_stopped(next_from_tile_x, next_from_tile_y);
                (next_from_tile_x, next_from_tile_y, next_move_dir)
            }
        };

        self.tile_movement.set_moving(next_move_dir);
        physics.set_tile_position(new_current_tile_x, new_current_tile_y, true);

        // This is important. Server has already applied velocity to the position, before sending the NTP, so we need to as well here.
        physics.step();

        physics.tick_log(update_tick, false);

        render_position.recv_position(physics.position(), update_tick);

        self.handle_late_update(
            tick_tracker,
            update_tick,
            physics,
            render_position,
            animation_state,
        );
    }

    // returns whether or not to rollback
    pub fn recv_updated_net_move_buffer(
        &mut self,
        tick_tracker: &TickTracker,
        update_tick: Tick,
        net_move_buffer: &NetworkedMoveBuffer,
        physics: &mut PhysicsController,
        render_position: &mut RenderPosition,
        animation_state: &mut AnimationState,
    ) -> bool {
        info!(
            "Recv NetworkedMoveBuffer. Tick: {:?}, Value: {:?}",
            update_tick,
            net_move_buffer.get()
        );

        let updated_value = net_move_buffer.get();
        if updated_value.is_none() {
            if self.move_buffer.has_buffered_move() {
                // changed
                self.move_buffer.clear();
                return true;
            } else {
                // did not change
                return false;
            }
        }

        let (updated_move_dir, updated_position, updated_velocity) = updated_value.unwrap();

        self.move_buffer.buffer_move(updated_move_dir);
        physics.set_position(updated_position.x, updated_position.y, true);
        physics.set_velocity(updated_velocity.x, updated_velocity.y, true);

        physics.tick_log(update_tick, false);

        render_position.recv_position(physics.position(), update_tick);

        self.handle_late_update(
            tick_tracker,
            update_tick,
            physics,
            render_position,
            animation_state,
        );

        return true;
    }

    fn handle_late_update(
        &mut self,
        tick_tracker: &TickTracker,
        update_tick: Tick,
        physics: &mut PhysicsController,
        render_position: &mut RenderPosition,
        animation_state: &mut AnimationState,
    ) {
        let Some(last_processed_server_tick) = tick_tracker.last_processed_server_tick() else {
            return;
        };
        if update_tick == last_processed_server_tick {
            return;
        }

        if sequence_greater_than(update_tick, last_processed_server_tick) {
            // if update_tick is more than last_processed_server_tick, then panic
            // TODO: do we need this? what to do here? is this possible?
            panic!(
                "Using last processed server tick: {:?}, instead of previous tick: {:?}",
                last_processed_server_tick, update_tick
            );
        }

        // if update_tick is less than last_processed_server_tick, then we should simulate forward
        let mut current_tick = update_tick;
        while current_tick != last_processed_server_tick {
            current_tick = current_tick.wrapping_add(1);
            process_tick(
                TileMovementType::ClientConfirmed,
                current_tick,
                None, // confirmed entities don't take commands
                self,
                physics,
                render_position,
                animation_state,
            );
        }
    }

    pub fn decompose_to_values(self) -> (TileMovement, MoveBuffer) {
        (self.tile_movement, self.move_buffer)
    }
}

fn pathfind_to_tile(ax: i16, ay: i16, bx: i16, by: i16) -> (i16, i16, Direction) {
    if ax == bx && ay == by {
        panic!("Unexpected! Shouldn't be the same tile");
    }
    info!(
        "Pathfinding from ({:?}, {:?}) to ({:?}, {:?})",
        ax, ay, bx, by
    );

    let mut lx = ax;
    let mut ly = ay;
    let mut dir = None;

    let mut cx = ax;
    let mut cy = ay;

    while cx != bx || cy != by {
        let dx = (bx - cx).min(1).max(-1);
        let dy = (by - cy).min(1).max(-1);

        dir = Direction::from_delta(dx as i8, dy as i8);
        if dir.is_none() {
            panic!("unexpected! shouldn't be allowed");
        }

        lx = cx;
        ly = cy;

        cx += dx;
        cy += dy;

        info!("Pathfinding: ({:?}, {:?})", cx, cy);
    }

    (lx, ly, dir.unwrap())
}
