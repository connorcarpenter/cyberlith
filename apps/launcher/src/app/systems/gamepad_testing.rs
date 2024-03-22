use std::time::Duration;

use bevy_ecs::{
    event::EventReader,
    system::ResMut,
};
use bevy_log::info;

use game_engine::input::{
    GamepadButtonType, GamepadRumbleIntensity, InputEvent,
    RumbleManager,
};

#[allow(unused)]
pub(crate) fn gamepad_testing_system(
    mut input_events: EventReader<InputEvent>,
    mut rumble_manager: ResMut<RumbleManager>,
) {
    // winit events
    for event in input_events.read() {
        match event {
            InputEvent::GamepadConnected(id) => {
                info!("INPUT: {:?} connected", id);
                info!("---");
            }
            InputEvent::GamepadDisconnected(id) => {
                info!("INPUT: {:?} disconnected", id);
                info!("---");
            }
            InputEvent::GamepadButtonPressed(id, btn) => {
                info!("INPUT: {:?} pressed", btn);
                info!("---");

                if *btn == GamepadButtonType::RightBumper {

                    // TESTING
                    let duration = 1000; //duration.as_millis() as u32;
                    let (weak, strong) = if game_engine::random::gen_bool() {
                        (0.0, 0.2)
                    } else {
                        (0.2, 0.0)
                    };
                    // TESTING

                    rumble_manager.add_rumble(
                        *id,
                        Duration::from_millis(duration),
                        GamepadRumbleIntensity {
                            strong_motor: strong,
                            weak_motor: weak,
                        },
                    );
                }
            }
            InputEvent::GamepadButtonReleased(_id, btn) => {
                info!("INPUT: {:?} released", btn);
                info!("---");
            }
            InputEvent::GamepadJoystickMoved(_id, joystick, value) => {
                info!("INPUT: {:?} move to {}", joystick, value);
                // info!("---");
            }
            _ => {}
        }
    }
}
