use bevy_ecs::{event::EventWriter, system::Res};

use game_engine::{
    input::{Input, Key},
    kernel::AppExitAction,
    logging::info,
};

pub fn process(input: Res<Input>, mut event_writer: EventWriter<AppExitAction>) {
    if input.is_pressed(Key::L) {
        info!("L pressed, going to launcher app");
        event_writer.send(AppExitAction::go_to("launcher"));
    }
}
