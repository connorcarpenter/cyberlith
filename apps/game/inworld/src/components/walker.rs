use bevy_ecs::component::Component;

use game_engine::asset::{AnimationData, AssetHandle};

#[derive(Component, Clone)]
pub struct WalkAnimation {
    pub(crate) anim_handle: AssetHandle<AnimationData>,
    pub(crate) animation_index_ms: f32,
}

impl WalkAnimation {
    pub fn new(anim_handle: AssetHandle<AnimationData>) -> Self {
        Self {
            anim_handle,
            animation_index_ms: 0.0,
        }
    }
}