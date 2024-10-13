use std::default::Default;

use bevy_ecs::{system::{Res, ResMut}, prelude::Resource};

use game_engine::{input::{Input}, naia::{GameInstant, CommandHistory, Tick}, world::{resources::{IncomingCommands, PlayerCommandEvent}, messages::{PlayerCommands}, WorldClient}};

use crate::resources::{Global, OutgoingCommands};

#[derive(Resource)]
pub struct InputManager {
    current_tick_opt: Option<Tick>,

    incoming_commands: IncomingCommands,
    outgoing_commands_opt: Option<OutgoingCommands>,

    command_history: CommandHistory<PlayerCommands>,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            current_tick_opt: None,
            incoming_commands: IncomingCommands::new(),
            outgoing_commands_opt: None,
            command_history: CommandHistory::default(),
        }
    }
}

impl InputManager {

    // used as a system
    pub fn recv_key_input(
        mut me: ResMut<Self>,
        client: WorldClient,
        global: Res<Global>,
        input: Res<Input>,
    ) {
        if global.owned_entity.is_none() {
            return;
        }

        let Some(client_instant) = client.client_instant() else {
            return;
        };

        if me.outgoing_commands_opt.is_none() {
            me.outgoing_commands_opt = Some(OutgoingCommands::new(client_instant));
        }
        let outgoing_commands = me.outgoing_commands_opt.as_mut().unwrap();
        outgoing_commands.recv_key_input(client_instant, &input);
    }

    pub fn pop_outgoing_command(&mut self, client_instant: GameInstant) -> Option<PlayerCommands> {
        self.outgoing_commands_opt.as_mut()?.pop_outgoing_command(client_instant)
    }

    pub fn save_to_command_history(&mut self, client_tick: Tick, command: &PlayerCommands) {
        {
            if !self.command_history.can_insert(&client_tick) {
                // History is full, should this be possible??
                panic!(
                    "Command History is full, cannot insert command for tick: {:?}",
                    client_tick
                );
            }

            // Record command
            self.command_history.insert(client_tick, command.clone());
        }
    }

    pub fn pop_command_replays(&mut self, server_tick: Tick) -> Vec<(Tick, PlayerCommands)> {

        // TODO: fix this?
        let modified_server_tick = server_tick.wrapping_sub(1);

        self.command_history.replays(&modified_server_tick)
    }

    pub fn recv_incoming_command(&mut self, tick: Tick, key_command_opt: Option<PlayerCommands>) {
        self.incoming_commands.recv_incoming_command(tick, key_command_opt);
    }

    pub fn pop_incoming_commands(&mut self, tick: Tick) -> Vec<PlayerCommandEvent> {
        self.incoming_commands.pop_incoming_commands(tick)
    }
}