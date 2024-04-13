use bevy_ecs::{system::Res, event::EventWriter};
use bevy_log::info;

use game_engine::{kernel::AppExitAction, input::{Input, Key}};

pub fn process(
    input: Res<Input>,
    mut event_writer: EventWriter<AppExitAction>,
) {
    if input.is_pressed(Key::L) {
        info!("L pressed, going to launcher app");
        event_writer.send(AppExitAction::go_to("launcher"));
    }
}