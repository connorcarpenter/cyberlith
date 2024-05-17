use bevy_ecs::{event::EventWriter, system::Res};

use game_engine::{
    input::{Input, Key},
    kernel::AppExitAction,
    logging::info,
    session::SessionClient,
};
use game_engine::session::{ClientActionsChannel, WorldConnectRequest};

pub fn process(
    input: Res<Input>,
    mut event_writer: EventWriter<AppExitAction>,
    mut session_client: SessionClient,
) {
    if input.is_pressed(Key::L) {
        info!("L pressed, going to launcher app");
        event_writer.send(AppExitAction::go_to("launcher"));
    }
    if input.is_pressed(Key::W) {
        info!("W pressed, connecting to world server");
        let message = WorldConnectRequest::new();
        session_client.send_message::<ClientActionsChannel, _>(&message);
    }
}
