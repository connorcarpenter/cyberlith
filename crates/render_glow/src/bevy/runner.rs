use std::default::Default;

use bevy_app::App;

use crate::{
    degrees, vec3,
    window::{FrameOutput, Window, WindowSettings},
    Camera, ClearState, Color, ColorMaterial, CpuMesh, FrameInput, Gm, Mesh, Positions,
};

#[derive(Default)]
pub struct ThreeDRunner {}

pub fn three_d_runner(mut app: App) {
    // Create a Window
    let window = Window::new(WindowSettings {
        title: "Triangle!".to_string(),
        max_size: Some((1280, 720)),
        ..Default::default()
    })
    .unwrap();

    // // Get the graphics context from the window
    // let context = window.gl();
    //
    // // Create a Camera
    // let mut camera = Camera::new_perspective(
    //     window.viewport(),
    //     vec3(0.0, 0.0, 2.0),
    //     vec3(0.0, 0.0, 0.0),
    //     vec3(0.0, 1.0, 0.0),
    //     degrees(45.0),
    //     0.1,
    //     10.0,
    // );
    //
    // // Create a CPU-side mesh consisting of a single colored triangle
    // let positions = vec![
    //     vec3(0.5, -0.5, 0.0),  // bottom right
    //     vec3(-0.5, -0.5, 0.0), // bottom left
    //     vec3(0.0, 0.5, 0.0),   // top
    // ];
    // let colors = vec![
    //     Color::new(255, 0, 0, 255), // bottom right
    //     Color::new(0, 255, 0, 255), // bottom left
    //     Color::new(0, 0, 255, 255), // top
    // ];
    // let cpu_mesh = CpuMesh {
    //     positions: Positions::F32(positions),
    //     colors: Some(colors),
    //     ..Default::default()
    // };
    //
    // // Construct a model, with a default color material, thereby transferring the mesh data to the GPU
    // let mut model = Gm::new(Mesh::new(&context, &cpu_mesh), ColorMaterial::default());

    // Start the main render loop
    window.render_loop(
        move |frame_input| // Begin a new frame with an updated frame input
            {
                // Insert FrameInput
                app
                    .world
                    .insert_non_send_resource(frame_input);

                // update app
                app.update();

                // Remove FrameInput
                app
                    .world
                    .remove_non_send_resource::<FrameInput<()>>();

                // // Ensure the viewport matches the current window viewport which changes if the window is resized
                // camera.set_viewport(frame_input.viewport);
                //
                // // Get the screen render target to be able to render something on the screen
                // frame_input.screen()
                //     // Clear the color and depth of the screen render target
                //     .clear(ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0))
                //     // Render the triangle with the color material which uses the per vertex colors defined at construction
                //     .render(
                //         &camera, &model, &[]
                //     );

                // Returns default frame output to end the frame
                FrameOutput::default()
            },
    );
}
