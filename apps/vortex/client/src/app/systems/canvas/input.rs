use bevy_ecs::system::{Query, ResMut};

use input::{Input, Key, MouseButton};
use render_api::components::{Camera, Projection, Transform};

use crate::app::resources::canvas_manager::{CanvasManager, ClickType};

pub fn input(
    mut canvas_manager: ResMut<CanvasManager>,
    mut input: ResMut<Input>,
    mut camera_query: Query<(&mut Camera, &mut Transform, &mut Projection)>,
) {
    // check keyboard input

    // (S)olid 3D View
    if input.is_pressed(Key::S) {
        // disable 2d camera, enable 3d camera
        canvas_manager.set_3d_mode(&mut camera_query);
    }
    // (W)ireframe 2D View
    else if input.is_pressed(Key::W) {
        // disable 3d camera, enable 2d camera
        canvas_manager.set_2d_mode(&mut camera_query);
    }
    // (G)ame Camera View
    else if input.is_pressed(Key::G) {
        canvas_manager.set_camera_angle_ingame();
    }
    // Si(d)e Camera View
    else if input.is_pressed(Key::D) {
        canvas_manager.set_camera_angle_side();
    }
    // (F)ront Camera View
    else if input.is_pressed(Key::F) {
        canvas_manager.set_camera_angle_front();
    }
    // (T)op Camera View
    else if input.is_pressed(Key::T) {
        canvas_manager.set_camera_angle_top();
    }

    // Mouse wheel zoom..
    let scroll_y = input.consume_mouse_scroll();
    if scroll_y > 0.1 || scroll_y < -0.1 {
        canvas_manager.camera_zoom(scroll_y);
    }

    // is a vertex currently selected?
    let vertex_is_selected = false;
    // is the cursor hovering over anything?
    let cursor_is_hovering = false;

    if !vertex_is_selected && !cursor_is_hovering {
        let left_button_pressed = input.is_pressed(MouseButton::Left);
        let right_button_pressed = input.is_pressed(MouseButton::Right);
        let mouse_button_pressed = left_button_pressed || right_button_pressed;

        if mouse_button_pressed {
            if left_button_pressed {
                canvas_manager.click_type = ClickType::Left;
            }
            if right_button_pressed {
                canvas_manager.click_type = ClickType::Right;
            }

            if canvas_manager.click_down {
                // already clicking
                let mouse = *input.mouse_position();
                let delta = mouse - canvas_manager.click_start;
                canvas_manager.click_start = mouse;

                if delta.length() > 0.0 {
                    match canvas_manager.click_type {
                        ClickType::Left => {
                            canvas_manager.camera_pan(delta);
                        }
                        ClickType::Right => {
                            canvas_manager.camera_orbit(delta);
                        }
                    }
                }
            } else {
                // haven't clicked yet
                canvas_manager.click_down = true;
                canvas_manager.click_start = *input.mouse_position();
            }
        } else {
            if canvas_manager.click_down {
                // release click
                canvas_manager.click_down = false;
            }
        }
    }
}