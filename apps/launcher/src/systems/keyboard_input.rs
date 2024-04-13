use bevy_ecs::{system::Res, event::EventWriter};
use bevy_log::info;

use game_engine::{kernel::AppExitAction, input::{Input, Key}};

pub fn process(
    input: Res<Input>,
    mut event_writer: EventWriter<AppExitAction>,
) {
    if input.is_pressed(Key::G) {
        info!("G pressed, going to game app");
        event_writer.send(AppExitAction::go_to("game"));
    }
}