use bevy_ecs::{
    entity::Entity,
    system::{Query, Res, ResMut},
};

use game_engine::{
    asset::{AssetHandle, AssetManager, AssetRender, UnitData},
    math::{Quat, Vec3},
    render::{
        components::{RenderLayer, Transform, Visibility},
        resources::RenderFrame,
    },
    time::Instant,
    world::WorldClient,
};

use crate::{
    components::{AnimationState, Confirmed, Predicted, RenderPosition},
    resources::Global,
};

pub fn draw_units(
    client: WorldClient,
    global: Res<Global>,
    asset_manager: Res<AssetManager>,
    mut render_frame: ResMut<RenderFrame>,
    mut unit_q: Query<(
        Entity,
        &AssetHandle<UnitData>,
        &mut AnimationState,
        // &RenderHelper,
        &mut RenderPosition,
        // &mut TileMovement,
        Option<&Confirmed>,
        Option<&Predicted>,
        &mut Transform,
        &Visibility,
        Option<&RenderLayer>,
    )>,
) {
    let now = Instant::now();

    // Aggregate Models
    for (
        entity,
        unit_handle,
        mut anim_state,
        // render_helper,
        mut render_position,
        // mut tile_movement,
        confirmed_opt,
        predicted_opt,
        mut transform,
        visibility,
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

        if confirmed_opt.is_some() && predicted_opt.is_some() {
            panic!("Unit cannot be both confirmed and predicted");
        }

        // let (current_position_x, current_position_y) = tile_movement.current_position();

        if predicted_opt.is_some() {
            // // draw predicted future queue
            // {
            //     for (future_tile_x, future_tile_y, _future_instant) in render_position.queue_ref().iter() {
            //         transform.translation.x = (*future_tile_x as f32);
            //         transform.translation.y = (*future_tile_y as f32) + (TILE_SIZE * 0.25);
            //         transform.set_scale(Vec3::new(5.0, 12.0, 20.0));
            //         render_frame.draw_mesh(
            //             render_layer_opt,
            //             &render_helper.cube_mesh_handle,
            //             &render_helper.yellow_mat_handle, // YELLOW = FUTURE
            //             &transform,
            //         );
            //     }
            // }

            // {
            //     // draw predicted interpolated position
            //     let (interp_x, interp_y) = render_position.render(&client, &now, true);
            //     transform.translation.x = interp_x;
            //     transform.translation.y = interp_y + (TILE_SIZE * 0.25);
            //     transform.set_scale(Vec3::new(22.0, 10.0, 5.0));
            //     render_frame.draw_mesh(
            //         render_layer_opt,
            //         &render_helper.cube_mesh_handle,
            //         &render_helper.green_mat_handle, // GREEN = INTERPOLATED
            //         &transform,
            //     );
            // }

            // draw predicted model // working!!!
            {
                let (interp_x, interp_y) = render_position.render(&client, &now);
                transform.translation.x = interp_x;
                transform.translation.y = interp_y;

                // TODO: put this in a system
                anim_state.update(&now, &asset_manager, animated_model_handle, &transform);

                transform.set_scale(Vec3::new(1.0, 1.0, 1.0));
                transform.set_rotation(Quat::from_rotation_z(anim_state.rotation));
                asset_manager.draw_animated_model(
                    &mut render_frame,
                    animated_model_handle,
                    &anim_state.animation_name,
                    &transform,
                    anim_state.animation_index_ms,
                    render_layer_opt,
                );
            }
        } else {
            // // check if this is ours, if so, we don't need to render it
            // if let Some(owned_entity) = &global.owned_entity {
            //     if owned_entity.confirmed == entity {
            //         continue;
            //     }
            // }

            // {
            //     // draw confirmed future queue
            //     let future_positions = render_position.queue_ref();
            //     for (future_tile_x, future_tile_y, _future_instant) in future_positions.iter() {
            //         transform.translation.x = (*future_tile_x as f32);
            //         transform.translation.y = (*future_tile_y as f32);
            //         transform.set_scale(Vec3::new(7.0, 10.0, 20.0));
            //         render_frame.draw_mesh(
            //             render_layer_opt,
            //             &render_helper.cube_mesh_handle,
            //             &render_helper.pink_mat_handle, // PINK = FUTURE CONFIRMED
            //             &transform,
            //         );
            //     }
            // }

            // {
            //     // draw confirmed interpolated position
            //     let (interp_x, interp_y) = render_position.render(&client, &now, false);
            //     transform.translation.x = interp_x;
            //     transform.translation.y = interp_y;
            //     transform.set_scale(Vec3::new(20.0, 12.0, 5.0));
            //     render_frame.draw_mesh(
            //         render_layer_opt,
            //         &render_helper.cube_mesh_handle,
            //         &render_helper.red_mat_handle, // RED = INTERPOLATED
            //         &transform,
            //     );
            // }

            // draw confirmed model // working!!!
            {
                let (interp_x, interp_y) = render_position.render(&client, &now);
                transform.translation.x = interp_x;
                transform.translation.y = interp_y;

                // TODO: put this in a system
                anim_state.update(&now, &asset_manager, animated_model_handle, &transform);

                transform.set_scale(Vec3::new(1.0, 1.0, 1.0));
                transform.set_rotation(Quat::from_rotation_z(anim_state.rotation));
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
    }
}
