
use bevy_ecs::component::Component;

use game_engine::asset::{AnimatedModelData, AssetHandle, AssetManager};
use game_engine::render::components::Transform;
use game_engine::time::Instant;

#[derive(Component, Clone)]
pub struct AnimationState {
    pub(crate) rotation: f32,
    pub(crate) animation_name: String,
    pub(crate) animation_index_ms: f32,
    last_now: Instant,
    last_pos: (f32, f32),
}

impl AnimationState {
    pub fn new() -> Self {
        Self {
            rotation: 0.0,
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
        // dx
        let (x, y) = (transform.translation.x, transform.translation.y);
        let (last_x, last_y) = self.last_pos;

        let dx = x - last_x;
        let dy = y - last_y;

        let rotation = dy.atan2(dx);

        self.last_pos = (x, y);

        // change animation if needed
        let is_moving = dx != 0.0 || dy != 0.0;
        let new_animation_name = if is_moving { "walk" } else { "idle" };
        if new_animation_name != self.animation_name {
            self.animation_name = new_animation_name.to_string();
            self.animation_index_ms = 0.0;
        }

        // change direction if needed
        if is_moving {
            self.rotation = rotation;
        }

        // animate
        let delta_ms = self.last_now.elapsed(now).as_millis();
        self.last_now = now.clone();

        let max_duration_ms = asset_manager.get_animated_model_animation_duration_ms(model_data, &self.animation_name);

        self.animation_index_ms += (delta_ms as f32) * 0.25;

        while self.animation_index_ms > max_duration_ms {
            self.animation_index_ms -= max_duration_ms;
        }
    }
}
