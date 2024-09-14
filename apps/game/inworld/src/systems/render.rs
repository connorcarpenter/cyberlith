use bevy_ecs::system::{Query, Res, ResMut};

use game_engine::{
    asset::{AssetHandle, AssetManager, AssetRender, UnitData},
    render::{
        components::{RenderLayer, Transform, Visibility},
        resources::RenderFrame,
    },
    world::{WorldClient, components::TileMovement},
    time::Instant,
};
use game_engine::math::Vec3;
use crate::components::{AnimationState, Confirmed, Predicted, RenderHelper, RenderPosition};

pub fn draw_units(
    // client: WorldClient,
    asset_manager: Res<AssetManager>,
    mut render_frame: ResMut<RenderFrame>,
    mut unit_q: Query<(
        &AssetHandle<UnitData>,
        // &AnimationState,
        &RenderHelper,
        &mut RenderPosition,
        &mut TileMovement,
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
        unit_handle,
        // anim_state,
        render_helper,
        mut render_position,
        mut tile_movement,
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
        // let Some(animated_model_handle) = asset_manager.get_unit_animated_model_handle(unit_handle)
        // else {
        //     continue;
        // };

        if confirmed_opt.is_some() && predicted_opt.is_some() {
            panic!("Unit cannot be both confirmed and predicted");
        }

        let (current_position_x, current_position_y) = tile_movement.current_position();
        transform.translation.x = current_position_x;
        transform.translation.y = current_position_y;

        if predicted_opt.is_some() {
            // // draw predicted position
            // {
            //     transform.set_scale(Vec3::new(10.0, 5.0, 20.0));
            //     render_frame.draw_mesh(
            //         render_layer_opt,
            //         &render_helper.cube_mesh_handle,
            //         &render_helper.aqua_mat_handle, // AQUA = PREDICTION
            //         &transform,
            //     );
            // }

            // draw predicted future queue
            {
                let future_positions = render_position.queue_ref();
                for (future_tile_x, future_tile_y, _future_instant) in future_positions.iter() {
                    transform.translation.x = (*future_tile_x as f32);
                    transform.translation.y = (*future_tile_y as f32);
                    transform.set_scale(Vec3::new(5.0, 10.0, 20.0));
                    render_frame.draw_mesh(
                        render_layer_opt,
                        &render_helper.cube_mesh_handle,
                        &render_helper.yellow_mat_handle, // YELLOW = FUTURE
                        &transform,
                    );
                }
            }

            {
                // draw predicted interpolated position
                let (interp_x, interp_y) = render_position.render(&now, true);
                transform.translation.x = interp_x;
                transform.translation.y = interp_y;
                transform.set_scale(Vec3::new(20.0, 10.0, 5.0));
                render_frame.draw_mesh(
                    render_layer_opt,
                    &render_helper.cube_mesh_handle,
                    &render_helper.green_mat_handle, // GREEN = INTERPOLATED
                    &transform,
                );
            }

            continue;
        }

        // draw model
        // asset_manager.draw_animated_model(
        //     &mut render_frame,
        //     animated_model_handle,
        //     &anim_state.animation_name,
        //     &transform,
        //     anim_state.animation_index_ms,
        //     render_layer_opt,
        // );

        {
            // draw confirmed position
            // transform.set_scale(Vec3::new(10.0, 20.0, 5.0));
            // render_frame.draw_mesh(
            //     render_layer_opt,
            //     &render_helper.cube_mesh_handle,
            //     &render_helper.blue_mat_handle, // BLUE = CONFIRMED
            //     &transform,
            // );
        }

        // {
        //     // draw confirmed future queue
        //     let future_positions = render_position.queue_ref();
        //     for (future_tile_x, future_tile_y, _future_instant) in future_positions.iter() {
        //         transform.translation.x = (*future_tile_x as f32);
        //         transform.translation.y = (*future_tile_y as f32);
        //         transform.set_scale(Vec3::new(5.0, 10.0, 20.0));
        //         render_frame.draw_mesh(
        //             render_layer_opt,
        //             &render_helper.cube_mesh_handle,
        //             &render_helper.yellow_mat_handle, // YELLOW = FUTURE
        //             &transform,
        //         );
        //     }
        // }

        {
            // draw confirmed interpolated position
            let (interp_x, interp_y) = render_position.render(&now, false);
            transform.translation.x = interp_x;
            transform.translation.y = interp_y;
            transform.set_scale(Vec3::new(20.0, 10.0, 5.0));
            render_frame.draw_mesh(
                render_layer_opt,
                &render_helper.cube_mesh_handle,
                &render_helper.red_mat_handle, // RED = INTERPOLATED
                &transform,
            );
        }
    }
}
