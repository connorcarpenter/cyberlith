use bevy_ecs::system::{Query, Res, ResMut};

use game_engine::{
    asset::{AssetHandle, AssetManager, AssetRender, ModelData},
    render::{
        components::{
            RenderLayer, Transform,
            Visibility,
        },
        resources::RenderFrame,
    },
};

use crate::inworld::systems::walker_scene::WalkAnimation;

pub fn draw(
    asset_manager: Res<AssetManager>,
    mut render_frame: ResMut<RenderFrame>,
    models_q: Query<(
        &AssetHandle<ModelData>,
        &WalkAnimation,
        &Transform,
        &Visibility,
        Option<&RenderLayer>,
    )>,
) {
    // Aggregate Models
    for (model_handle, walk_anim, transform, visibility, render_layer_opt) in models_q.iter() {
        if !visibility.visible {
            continue;
        }
        asset_manager.draw_animated_model(
            &mut render_frame,
            model_handle,
            &walk_anim.anim_handle,
            transform,
            walk_anim.animation_index_ms,
            render_layer_opt,
        );
    }
}
