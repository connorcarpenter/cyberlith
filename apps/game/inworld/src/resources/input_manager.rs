use std::{default::Default, collections::{HashMap, VecDeque}};

use bevy_ecs::{system::{Res, ResMut}, prelude::Resource};

use game_engine::{input::{Key, Input}, naia::{GameInstant, CommandHistory, Tick}, world::{messages::PlayerCommands, types::Direction, WorldClient}};

use crate::resources::Global;

#[derive(Resource)]
pub struct InputManager {
    tracked_keys: Vec<Key>,
    pressed_keys: HashMap<Key, (GameInstant, u16)>,

    next_command: Option<PlayerCommands>,
    incoming_commands: VecDeque<(Tick, Option<PlayerCommands>)>,
    command_history: CommandHistory<Option<PlayerCommands>>,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            tracked_keys: vec![
                Key::W, Key::S, Key::A, Key::D,
            ],
            pressed_keys: HashMap::new(),
            next_command: None,
            incoming_commands: VecDeque::new(),
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

        // keep track of pressed keys & durations
        let releases = me.update_pressed_keys(client_instant, &input);

        // modify playercommand as needed
        me.update_player_command(releases);
    }

    fn update_pressed_keys(&mut self, client_instant: GameInstant, input: &Input) -> Vec<(Key, u16)> {
        let mut output = Vec::new();
        for key in &self.tracked_keys {
            if input.is_pressed(*key) {
                if let Some((prev_instant, prev_duration)) = self.pressed_keys.get_mut(key) {
                    let duration = prev_instant.offset_from(&client_instant);
                    *prev_instant = client_instant;
                    if *prev_duration + duration as u16 > 1000 {
                        *prev_duration = 1000;
                    } else {
                        *prev_duration += duration as u16;
                    }
                } else {
                    self.pressed_keys.insert(*key, (client_instant, 1));
                }
            } else {
                if let Some((prev_instant, prev_duration)) = self.pressed_keys.remove(key) {
                    let duration = prev_instant.offset_from(&client_instant);
                    if prev_duration + duration as u16 > 1000 {
                        output.push((*key, 1000));
                    } else {
                        output.push((*key, prev_duration + duration as u16));
                    }
                }
            }
        }

        output
    }

    fn update_player_command(&mut self, releases: Vec<(Key, u16)>) {
        if self.pressed_keys.is_empty() && releases.is_empty() {
            return;
        }

        if self.next_command.is_none() {
            self.next_command = Some(PlayerCommands::new());
        }

        let next_command = self.next_command.as_mut().unwrap();

        let mut hold_u = false;
        let mut hold_d = false;
        let mut hold_l = false;
        let mut hold_r = false;

        let mut tap_u = false;
        let mut tap_d = false;
        let mut tap_l = false;
        let mut tap_r = false;
        for (key, duration) in releases {
            if duration < 150 {
                // this was a tap
                match key {
                    Key::W => tap_u = true,
                    Key::S => tap_d = true,
                    Key::A => tap_l = true,
                    Key::D => tap_r = true,
                    _ => {},
                }
            } else {
                match key {
                    Key::W => hold_u = true,
                    Key::S => hold_d = true,
                    Key::A => hold_l = true,
                    Key::D => hold_r = true,
                    _ => {},
                }
            }
        }
        if !hold_u {
            if let Some((_, duration)) = self.pressed_keys.get(&Key::W) {
                if *duration >= 150 {
                    hold_u = true;
                }
            }
        }
        if !hold_d {
            if let Some((_, duration)) = self.pressed_keys.get(&Key::S) {
                if *duration >= 150 {
                    hold_d = true;
                }
            }
        }
        if !hold_l {
            if let Some((_, duration)) = self.pressed_keys.get(&Key::A) {
                if *duration >= 150 {
                    hold_l = true;
                }
            }
        }
        if !hold_r {
            if let Some((_, duration)) = self.pressed_keys.get(&Key::D) {
                if *duration >= 150 {
                    hold_r = true;
                }
            }
        }

        if tap_u && tap_d {
            tap_u = false;
            tap_d = false;
        }
        if tap_l && tap_r {
            tap_l = false;
            tap_r = false;
        }
        if hold_u && hold_d {
            hold_u = false;
            hold_d = false;
        }
        if hold_l && hold_r {
            hold_l = false;
            hold_r = false;
        }

        // Looking
        match (tap_u, tap_d, tap_l, tap_r) {
            (false, false, false, true ) => next_command.set_look(Direction::East),
            (true , false, false, true ) => next_command.set_look(Direction::Northeast),
            (true , false, false, false) => next_command.set_look(Direction::North),
            (true , false, true , false) => next_command.set_look(Direction::Northwest),
            (false, false, true , false) => next_command.set_look(Direction::West),
            (false, true , true , false) => next_command.set_look(Direction::Southwest),
            (false, true , false, false) => next_command.set_look(Direction::South),
            (false, true , false, true ) => next_command.set_look(Direction::Southeast),
            (false, false, false, false) => {},
            _ => panic!("Invalid look command"),
        }

        // Movement
        match (hold_u, hold_d, hold_l, hold_r) {
            (false, false, false, true ) => next_command.set_move(Direction::East),
            (true , false, false, true ) => next_command.set_move(Direction::Northeast),
            (true , false, false, false) => next_command.set_move(Direction::North),
            (true , false, true , false) => next_command.set_move(Direction::Northwest),
            (false, false, true , false) => next_command.set_move(Direction::West),
            (false, true , true , false) => next_command.set_move(Direction::Southwest),
            (false, true , false, false) => next_command.set_move(Direction::South),
            (false, true , false, true ) => next_command.set_move(Direction::Southeast),
            (false, false, false, false) => {},
            _ => panic!("Invalid move command"),
        }
    }

    pub fn pop_outgoing_command(&mut self) -> Option<PlayerCommands> {
        self.next_command.take()
    }

    pub fn save_to_command_history(&mut self, client_tick: Tick, command_opt: Option<PlayerCommands>) {
        {
            if !self.command_history.can_insert(&client_tick) {

                let most_recent_command_tick = self.command_history.most_recent_tick().unwrap();

                // History is full, should this be possible??
                panic!(
                    "Command History is full, cannot insert command for tick: {:?}, (most recent tick is {:?})",
                    client_tick,
                    most_recent_command_tick,
                );
            }

            // Record command
            self.command_history.insert(client_tick, command_opt.clone());
        }
    }

    pub fn pop_command_replays(&mut self, server_tick: Tick) -> Vec<(Tick, Option<PlayerCommands>)> {

        // TODO: fix this?
        let modified_server_tick = server_tick.wrapping_sub(1);

        self.command_history.replays(&modified_server_tick)
    }

    pub fn recv_incoming_command(&mut self, tick: Tick, key_command_opt: Option<PlayerCommands>) {
        self.incoming_commands.push_back((tick, key_command_opt));
    }

    pub fn pop_incoming_command(&mut self, tick: Tick) -> Option<PlayerCommands> {
        let (command_tick, command_opt) = self.incoming_commands.pop_front().unwrap();
        if command_tick != tick {
            panic!("Command tick mismatch");
        }
        command_opt
    }
}