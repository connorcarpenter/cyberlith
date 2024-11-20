use bevy_ecs::component::Component;

use game_engine::{
    asset::{AnimatedModelData, AssetHandle, AssetManager},
    time::Instant,
    world::types::Direction,
};
use game_engine::logging::info;
use game_engine::math::Vec2;

#[derive(Component, Clone)]
pub struct AnimationState {
    pub(crate) rotation: f32,
    lookdir: Direction,
    pub(crate) animation_name: String,
    pub(crate) animation_index_ms: f32,
    last_now: Instant,
    last_pos: (f32, f32),
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
            last_now: Instant::now(),
            last_pos: (0.0, 0.0),
            is_moving: false,
            move_heat: 0.0,
        }
    }

    pub fn update(
        &mut self,
        now: &Instant,
        asset_manager: &AssetManager,
        model_data: &AssetHandle<AnimatedModelData>,
        position_x: f32,
        position_y: f32,
        velocity_x: f32,
        velocity_y: f32,
    ) {
        let (last_x, last_y) = self.last_pos;
        self.last_pos = (position_x, position_y);

        let dx = position_x - last_x;
        let dy = position_y - last_y;

        // change animation if needed
        let is_moving = dx != 0.0 || dy != 0.0;

        if is_moving != self.is_moving {
            self.move_heat += 1.0;
            if self.move_heat > 7.0 {
                self.move_heat = 0.0;
                self.is_moving = is_moving;

                let new_animation_name = if self.is_moving { "walk" } else { "idle" };

                info!("Changing animation to: {} .. dx: {}, dy: {}", new_animation_name, dx, dy);
                self.animation_name = new_animation_name.to_string();
            }
        }

        // change direction if needed
        if self.is_moving && is_moving {
            let velocity = Vec2::new(velocity_x, velocity_y);
            if velocity.length() > 0.1 {
                self.lookdir = Direction::from_coords(velocity_x, velocity_y);
                self.rotation = self.lookdir.to_radians();
            }
        }

        // animate
        let delta_ms = self.last_now.elapsed(now).as_millis(); // TODO: delta should be some global thats passed into here...
        self.last_now = now.clone();

        // TODO: move this into config
        let animation_speed_factor = match self.animation_name.as_str() {
            "idle" => 0.075,
            "walk" => {
                let distance = Vec2::new(dx, dy).length();
                0.15 * distance
            },
            _ => 0.0,
        };
        self.animation_index_ms += (delta_ms as f32) * animation_speed_factor;

        let max_duration_ms = asset_manager
            .get_animated_model_animation_duration_ms(model_data, &self.animation_name);

        while self.animation_index_ms >= max_duration_ms {
            self.animation_index_ms -= max_duration_ms;
        }
    }

    pub fn recv_rollback(&mut self, other: &AnimationState) {
        // self.rotation = other.rotation;
        self.lookdir = other.lookdir;
        // self.animation_name = other.animation_name.clone();
        // self.animation_index_ms = other.animation_index_ms;
        // TODO: should we rollback other props?
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
