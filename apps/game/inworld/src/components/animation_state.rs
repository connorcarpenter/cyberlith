use bevy_ecs::component::Component;

use game_engine::{
    asset::{AnimatedModelData, AssetHandle, AssetManager},
    logging::info,
    math::Vec2,
};

use game_app_network::world::{components::TileMovement, types::Direction};

#[derive(Component, Clone)]
pub struct AnimationState {
    pub(crate) rotation: f32,
    lookdir: Direction,
    pub(crate) animation_name: String,
    pub(crate) animation_index_ms: f32,
    last_pos: Vec2,
    is_moving: bool,
    move_heat: f32,
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            rotation: 0.0,
            lookdir: Direction::East,
            animation_name: "idle".to_string(),
            animation_index_ms: 0.0,
            last_pos: Vec2::ZERO,
            is_moving: false,
            move_heat: 0.0,
        }
    }

    pub fn update(
        &mut self,
        asset_manager: &AssetManager,
        model_data: &AssetHandle<AnimatedModelData>,
        position: Vec2,
        velocity: Vec2,
        acceleration: Vec2,
        delta_ms: f32,
        tile_movement: &TileMovement,
    ) {
        let last_position = self.last_pos;
        self.last_pos = position;

        let dx = position.x - last_position.x;
        let dy = position.y - last_position.y;

        // change animation if needed
        let is_moving = dx != 0.0 || dy != 0.0;

        if is_moving != self.is_moving {
            self.move_heat += 1.0;
            if self.move_heat > 7.0 {
                self.move_heat = 0.0;
                self.is_moving = is_moving;

                let new_animation_name = if self.is_moving { "walk" } else { "idle" };

                info!(
                    "Changing animation to: {} .. dx: {}, dy: {}",
                    new_animation_name, dx, dy
                );
                self.animation_name = new_animation_name.to_string();
            }
        }

        // change direction if needed
        if self.is_moving && is_moving {
            // if acceleration.length() > 0.5 {
            //     self.lookdir = Direction::from_coords(acceleration.x, acceleration.y);
            // }
            if tile_movement.is_moving() {
                self.lookdir = tile_movement.as_moving().direction();
            }
            self.rotation = self.lookdir.to_radians();
        }

        // animate

        // TODO: move this into config
        let animation_speed_factor = match self.animation_name.as_str() {
            "idle" => 0.075,
            "walk" => {
                let distance = Vec2::new(dx, dy).length();
                0.15 * distance
            }
            _ => 0.0,
        };
        self.animation_index_ms += delta_ms * animation_speed_factor;

        let max_duration_ms = asset_manager
            .get_animated_model_animation_duration_ms(model_data, &self.animation_name);

        while self.animation_index_ms >= max_duration_ms {
            self.animation_index_ms -= max_duration_ms;
        }
    }

    pub fn recv_lookdir_update(&mut self, lookdir: &Direction) {
        if self.animation_name == "idle" {
            self.rotation = lookdir.to_radians();
            self.lookdir = *lookdir;
        }
    }

    pub fn lookdir(&self) -> Direction {
        self.lookdir
    }
}
