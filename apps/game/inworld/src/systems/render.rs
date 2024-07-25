use bevy_ecs::system::{Query, Res, ResMut};

use game_engine::{
    asset::{AssetHandle, AssetManager, AssetRender, UnitData},
    render::{
        components::{
            RenderLayer, Transform,
            Visibility,
        },
        resources::RenderFrame,
    },
};

use crate::components::AnimationState;

pub fn draw_units(
    asset_manager: Res<AssetManager>,
    mut render_frame: ResMut<RenderFrame>,
    unit_q: Query<(
        &AssetHandle<UnitData>,
        &AnimationState,
        &Transform,
        &Visibility,
        Option<&RenderLayer>,
    )>,
) {
    // Aggregate Models
    for (
        unit_handle,
        anim_state,
        transform,
        visibility,
        render_layer_opt
    ) in unit_q.iter() {
        if !visibility.visible {
            continue;
        }
        let Some(animated_model_handle) = asset_manager.get_unit_animated_model_handle(unit_handle) else {
            continue;
        };

        asset_manager.draw_animated_model(
            &mut render_frame,
            animated_model_handle,
            &anim_state.animation_name,
            transform,
            anim_state.animation_index_ms,
            render_layer_opt,
        );
    }
}
