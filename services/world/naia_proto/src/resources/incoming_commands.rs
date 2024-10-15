use std::collections::HashSet;

use naia_bevy_shared::Tick;
use logging::info;
use crate::{resources::CommandTimeline, messages::{PlayerCommands, PlayerCommandStream, PlayerCommand}};

const TICK_DURATION_MS: u16 = 40; // TODO: move to config

// Intended to be encapsulated within a Client or Server specific Resource!
pub struct IncomingCommands {
    map: HashSet<PlayerCommand>,
}

impl IncomingCommands {
    pub fn new() -> Self {

        let mut map = HashSet::new();
        map.insert(PlayerCommand::Up);
        map.insert(PlayerCommand::Down);
        map.insert(PlayerCommand::Left);
        map.insert(PlayerCommand::Right);

        Self {
            map
        }
    }

    pub fn recv_incoming_command(&mut self, tick: Tick, player_commands_opt: Option<PlayerCommands>) -> CommandTimeline {

        let mut output = CommandTimeline::new(tick);

        let Some(player_commands) = player_commands_opt.as_ref() else {
            return output;
        };

        for command in self.map.iter() {
            if let Some(command_stream) = player_commands.get(command) {
                let stream_output = Self::recv_stream(command_stream);
                output.recv_stream(*command, stream_output);
            }
        }

        output
    }

    fn recv_stream(key_stream: &PlayerCommandStream) -> (Option<u16>, Vec<(bool, u8)>) {
        info!("recv_stream: {:?}", key_stream);
        let start_pressed: Option<u16> = key_stream.start_pressed().map(|x| x.get() as u16);
        let mut pressed = start_pressed.is_some();
        let incoming_durations = key_stream.durations();
        let mut remaining_duration = TICK_DURATION_MS as u8;
        let mut outgoing_durations = Vec::new();

        for duration in incoming_durations {
            let duration = duration.get() as u8;
            if duration > remaining_duration {
                panic!("duration > remaining_duration!");
            } else {
                outgoing_durations.push((pressed, duration));
                pressed = !pressed;
                remaining_duration -= duration;
            }
        }

        if remaining_duration > 0 {
            outgoing_durations.push((pressed, remaining_duration));
        }

        return (start_pressed, outgoing_durations);
    }
}