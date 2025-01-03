use bevy_ecs::{
    entity::Entity,
    system::{Query, Res, ResMut, SystemState},
};

use game_engine::{
    asset::{AssetHandle, AssetManager, AssetRender, UnitData},
    math::{Quat, Vec3},
    render::{
        components::{RenderLayer, Transform, Visibility},
        resources::{RenderFrame, Time},
    },
};

use game_app_network::world::WorldClient;

use crate::{
    components::{AnimationState, RenderPosition},
    resources::PredictedWorld,
};

pub fn draw_units(
    time: Res<Time>,
    client: WorldClient,
    asset_manager: Res<AssetManager>,
    mut render_frame: ResMut<RenderFrame>,
    mut predicted_world: ResMut<PredictedWorld>,
    mut unit_q: Query<(
        Entity,
        &AssetHandle<UnitData>,
        &Visibility,
        &AnimationState,
        &mut RenderPosition,
        Option<&RenderLayer>,
    )>,
) {
    let duration_ms = time.get_elapsed_ms();

    let mut predicted_system_state: SystemState<Query<(&AnimationState, &mut RenderPosition)>> =
        SystemState::new(predicted_world.world_mut());
    let mut predicted_unit_q = predicted_system_state.get_mut(predicted_world.world_mut());

    let mut transform = Transform::default();

    // Aggregate Models
    for (
        entity,
        unit_handle,
        visibility,
        confirmed_anim_state,
        mut confirmed_render_position,
        render_layer_opt,
    ) in unit_q.iter_mut()
    {
        if !visibility.visible {
            continue;
        }
        let Some(animated_model_handle) = asset_manager.get_unit_animated_model_handle(unit_handle)
        else {
            continue;
        };

        let Ok((predicted_anim_state, mut predicted_render_position)) =
            predicted_unit_q.get_mut(entity)
        else {
            continue;
        };

        // draw confirmed model
        {
            let interp_position = confirmed_render_position.render(&client, duration_ms);

            transform.translation.x = interp_position.x;
            transform.translation.y = interp_position.y;
            transform.set_scale(Vec3::new(1.0, 1.0, 1.0));
            transform.set_rotation(Quat::from_rotation_z(predicted_anim_state.rotation));

            asset_manager.draw_animated_model(
                &mut render_frame,
                animated_model_handle,
                &confirmed_anim_state.animation_name,
                &transform,
                confirmed_anim_state.animation_index_ms,
                render_layer_opt,
            );
        }

        // draw predicted model
        {
            let interp_position = predicted_render_position.render(&client, duration_ms);

            transform.translation.x = interp_position.x;
            transform.translation.y = interp_position.y;
            transform.set_scale(Vec3::new(1.0, 1.0, 1.0));
            transform.set_rotation(Quat::from_rotation_z(predicted_anim_state.rotation));

            asset_manager.draw_animated_model(
                &mut render_frame,
                animated_model_handle,
                &predicted_anim_state.animation_name,
                &transform,
                predicted_anim_state.animation_index_ms,
                render_layer_opt,
            );
        }

        // if predicted_opt.is_some() {
        //     // draw predicted future queue
        //     {
        //         for (future_tile_x, future_tile_y, _future_instant) in render_position.queue_ref().iter() {
        //             transform.translation.x = (*future_tile_x as f32);
        //             transform.translation.y = (*future_tile_y as f32) + (TILE_SIZE * 0.25);
        //             transform.set_scale(Vec3::new(5.0, 12.0, 20.0));
        //             render_frame.draw_mesh(
        //                 render_layer_opt,
        //                 &render_helper.cube_mesh_handle,
        //                 &render_helper.yellow_mat_handle, // YELLOW = FUTURE
        //                 &transform,
        //             );
        //         }
        //     }
        //
        //     {
        //         // draw predicted interpolated position
        //         let (interp_x, interp_y) = render_position.render(&client, &now, true);
        //         transform.translation.x = interp_x;
        //         transform.translation.y = interp_y + (TILE_SIZE * 0.25);
        //         transform.set_scale(Vec3::new(22.0, 10.0, 5.0));
        //         render_frame.draw_mesh(
        //             render_layer_opt,
        //             &render_helper.cube_mesh_handle,
        //             &render_helper.green_mat_handle, // GREEN = INTERPOLATED
        //             &transform,
        //         );
        //     }
        // } else {
        //     // check if this is ours, if so, we don't need to render it
        //     if let Some(owned_entity) = &global.owned_entity {
        //         if owned_entity.confirmed == entity {
        //             continue;
        //         }
        //     }
        //
        //     {
        //         // draw confirmed future queue
        //         let future_positions = render_position.queue_ref();
        //         for (future_tile_x, future_tile_y, _future_instant) in future_positions.iter() {
        //             transform.translation.x = (*future_tile_x as f32);
        //             transform.translation.y = (*future_tile_y as f32);
        //             transform.set_scale(Vec3::new(7.0, 10.0, 20.0));
        //             render_frame.draw_mesh(
        //                 render_layer_opt,
        //                 &render_helper.cube_mesh_handle,
        //                 &render_helper.pink_mat_handle, // PINK = FUTURE CONFIRMED
        //                 &transform,
        //             );
        //         }
        //     }
        //
        //     {
        //         // draw confirmed interpolated position
        //         let (interp_x, interp_y) = render_position.render(&client, &now, false);
        //         transform.translation.x = interp_x;
        //         transform.translation.y = interp_y;
        //         transform.set_scale(Vec3::new(20.0, 12.0, 5.0));
        //         render_frame.draw_mesh(
        //             render_layer_opt,
        //             &render_helper.cube_mesh_handle,
        //             &render_helper.red_mat_handle, // RED = INTERPOLATED
        //             &transform,
        //         );
        //     }
        // }
    }
}
