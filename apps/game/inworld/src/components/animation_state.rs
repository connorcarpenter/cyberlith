use bevy_ecs::component::Component;

use game_engine::{
    asset::{AnimatedModelData, AssetHandle, AssetManager},
    render::components::Transform,
    time::Instant,
    world::types::Direction,
};

#[derive(Component, Clone)]
pub struct AnimationState {
    pub(crate) rotation: f32,
    lookdir: Direction,
    pub(crate) animation_name: String,
    pub(crate) animation_index_ms: f32,
    last_now: Instant,
    last_pos: (f32, f32),
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
        }
    }

    pub fn update(
        &mut self,
        now: &Instant,
        asset_manager: &AssetManager,
        model_data: &AssetHandle<AnimatedModelData>,
        transform: &Transform,
    ) {
        let (x, y) = (transform.translation.x, transform.translation.y);
        let (last_x, last_y) = self.last_pos;
        self.last_pos = (x, y);

        let dx = x - last_x;
        let dy = y - last_y;

        // change animation if needed
        let is_moving = dx != 0.0 || dy != 0.0;
        let new_animation_name = if is_moving { "walk" } else { "idle" };
        if new_animation_name != self.animation_name {
            self.animation_name = new_animation_name.to_string();
        }

        // change direction if needed
        if is_moving {
            self.lookdir = Direction::from_coords(dx as f32, dy as f32);
            self.rotation = self.lookdir.to_radians();
        }

        // animate
        let delta_ms = self.last_now.elapsed(now).as_millis(); // TODO: delta should be some global thats passed into here...
        self.last_now = now.clone();

        // TODO: move this into config
        let animation_speed_factor = match self.animation_name.as_str() {
            "idle" => 0.075,
            "walk" => 0.65,
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
        self.rotation = other.rotation;
        self.lookdir = other.lookdir;
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
