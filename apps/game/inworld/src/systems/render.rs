use bevy_ecs::system::{Query, Res, ResMut};

use game_engine::{
    world::WorldClient,
    asset::{AssetHandle, AssetManager, AssetRender, UnitData},
    render::{
        components::{
            RenderLayer, Transform,
            Visibility,
        },
        resources::RenderFrame,
    },
};

use crate::components::{AnimationState, Confirmed, Interp, Predicted};

pub fn draw_units(
    client: WorldClient,
    asset_manager: Res<AssetManager>,
    mut render_frame: ResMut<RenderFrame>,
    mut unit_q: Query<(
        &AssetHandle<UnitData>,
        &AnimationState,
        &Interp,
        Option<&Confirmed>,
        Option<&Predicted>,
        &mut Transform,
        &Visibility,
        Option<&RenderLayer>,
    )>,
) {
    // Aggregate Models
    for (
        unit_handle,
        anim_state,
        interp,
        confirmed_opt,
        predicted_opt,
        mut transform,
        visibility,
        render_layer_opt
    ) in unit_q.iter_mut() {
        if !visibility.visible {
            continue;
        }
        let Some(animated_model_handle) = asset_manager.get_unit_animated_model_handle(unit_handle) else {
            continue;
        };

        if confirmed_opt.is_some() && predicted_opt.is_some() {
            panic!("Unit cannot be both confirmed and predicted");
        }

        let interpolation = if confirmed_opt.is_some() {
            client.server_interpolation().unwrap()
        } else if predicted_opt.is_some() {
            client.client_interpolation().unwrap()
        } else {
            panic!("Unit must be either confirmed or predicted");
        };

        let (interp_x, interp_y) = interp.interpolate(interpolation);
        transform.translation.x = interp_x;
        transform.translation.y = interp_y;

        asset_manager.draw_animated_model(
            &mut render_frame,
            animated_model_handle,
            &anim_state.animation_name,
            &transform,
            anim_state.animation_index_ms,
            render_layer_opt,
        );
    }
}
