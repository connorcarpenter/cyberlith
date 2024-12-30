use std::{
    collections::{HashMap, VecDeque},
    default::Default,
};

use bevy_ecs::{
    prelude::Resource,
    system::{Query, Res, ResMut, SystemState},
};

use game_engine::{
    input::{Input, Key},
    logging::{info, warn},
};

use game_app_network::{
    naia::{CommandHistory, GameInstant, Tick},
    world::{messages::PlayerCommands, types::Direction, WorldClient},
};

use crate::{components::AnimationState, resources::{Global, PredictedWorld}};

const DOUBLE_TAP_BUFFER: u16 = 150;
const SEQUENTIAL_TAP_DURATION: u16 = 1000;
const MAX_TAP_DURATION: u16 = 180;
const MAX_HOLD_DURATION: u16 = 1000;

#[derive(Resource)]
pub struct InputManager {
    tracked_keys: Vec<Key>,
    pressed_keys: HashMap<Key, (GameInstant, u16)>,

    // up, down, left, right
    double_tap: Option<(GameInstant, bool, bool, bool, bool)>,
    sequential_tap_instant: Option<GameInstant>,

    next_command: Option<PlayerCommands>,
    incoming_commands: VecDeque<(Tick, Option<PlayerCommands>)>,
    command_history: CommandHistory<Option<PlayerCommands>>,
}

impl Default for InputManager {
    fn default() -> Self {
        Self {
            tracked_keys: vec![Key::W, Key::S, Key::A, Key::D],
            pressed_keys: HashMap::new(),
            next_command: None,
            double_tap: None,
            sequential_tap_instant: None,
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
        mut predicted_world: ResMut<PredictedWorld>,
    ) {
        if global.owned_entity.is_none() {
            return;
        }

        let Some(client_instant) = client.client_instant() else {
            return;
        };

        // keep track of pressed keys & durations
        let releases = me.update_pressed_keys(&client_instant, &input);

        // get prediction's current look direction
        let client_avatar_entity = global.owned_entity.as_ref().unwrap().confirmed;

        let mut predicted_system_state: SystemState<Query<&AnimationState>> = SystemState::new(predicted_world.world_mut());
        let animation_state_q = predicted_system_state.get(predicted_world.world_mut());
        let Ok(animation_state) = animation_state_q.get(client_avatar_entity) else {
            return;
        };
        let lookdir = animation_state.lookdir();

        // modify playercommand as needed
        me.update_player_command(&client_instant, releases, lookdir);
    }

    fn update_pressed_keys(
        &mut self,
        client_instant: &GameInstant,
        input: &Input,
    ) -> Vec<(Key, u16)> {
        let mut output = Vec::new();

        for key in &self.tracked_keys {
            if input.is_pressed(*key) {
                if let Some((prev_instant, prev_duration)) = self.pressed_keys.get_mut(key) {
                    *prev_duration =
                        Self::get_hold_duration(prev_instant, client_instant, *prev_duration);
                    *prev_instant = *client_instant;
                } else {
                    self.pressed_keys.insert(*key, (*client_instant, 1));
                }
            } else {
                if let Some((prev_instant, prev_duration)) = self.pressed_keys.remove(key) {
                    let duration =
                        Self::get_hold_duration(&prev_instant, client_instant, prev_duration);
                    output.push((*key, duration));
                }
            }
        }

        output
    }

    fn get_hold_duration(
        prev_instant: &GameInstant,
        client_instant: &GameInstant,
        prev_duration: u16,
    ) -> u16 {
        let mut duration = prev_instant.offset_from(client_instant);
        if duration < 0 {
            warn! {"How can duration of keypress be < 0?? Prev: {:?} -> Cur: {:?} = Dur {:?}", prev_instant, client_instant, duration};
            duration = 0;
        }
        if duration > MAX_HOLD_DURATION as i32 {
            duration = MAX_HOLD_DURATION as i32;
        }
        let duration = duration as u16;
        if prev_duration + duration > MAX_HOLD_DURATION {
            return MAX_HOLD_DURATION;
        } else {
            return prev_duration + duration;
        }
    }

    fn update_player_command(
        &mut self,
        client_instant: &GameInstant,
        releases: Vec<(Key, u16)>,
        current_lookdir: Direction,
    ) {
        if self.pressed_keys.is_empty() && releases.is_empty() {
            return;
        }

        if self.next_command.is_none() {
            self.next_command = Some(PlayerCommands::new());
        }

        self.handle_taps(client_instant, releases, current_lookdir);
        self.handle_holds();
    }

    fn handle_taps(
        &mut self,
        client_instant: &GameInstant,
        releases: Vec<(Key, u16)>,
        lookdir: Direction,
    ) {
        let next_command = self.next_command.as_mut().unwrap();

        if let Some((last_instant, _, _, _, _)) = &self.double_tap {
            let duration = last_instant.offset_from(client_instant) as u16;
            if duration > DOUBLE_TAP_BUFFER {
                self.double_tap = None;
            }
        }
        if let Some(last_instant) = &self.sequential_tap_instant {
            let duration = last_instant.offset_from(client_instant) as u16;
            if duration > SEQUENTIAL_TAP_DURATION {
                self.sequential_tap_instant = None;
            }
        }

        let mut tap_u = false;
        let mut tap_d = false;
        let mut tap_l = false;
        let mut tap_r = false;
        for (key, duration) in releases {
            if duration < MAX_TAP_DURATION {
                // this was a tap
                match key {
                    Key::W => tap_u = true,
                    Key::S => tap_d = true,
                    Key::A => tap_l = true,
                    Key::D => tap_r = true,
                    _ => {}
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

        if tap_u || tap_d || tap_l || tap_r {
            if let Some((_, last_tap_u, last_tap_d, last_tap_l, last_tap_r)) =
                self.double_tap.take()
            {
                // this is a hack to make it simpler to handle all the conditions
                if (last_tap_u && last_tap_l)
                    || (last_tap_u && last_tap_r)
                    || (last_tap_d && last_tap_l)
                    || (last_tap_d && last_tap_r)
                    || (tap_u && tap_l)
                    || (tap_u && tap_r)
                    || (tap_d && tap_l)
                    || (tap_d && tap_r)
                {
                    // skip
                } else {
                    if (last_tap_u && tap_l) || (last_tap_l && tap_u) {
                        tap_u = true;
                        tap_l = true;
                    } else if (last_tap_u && tap_r) || (last_tap_r && tap_u) {
                        tap_u = true;
                        tap_r = true;
                    } else if (last_tap_d && tap_l) || (last_tap_l && tap_d) {
                        tap_d = true;
                        tap_l = true;
                    } else if (last_tap_d && tap_r) || (last_tap_r && tap_d) {
                        tap_d = true;
                        tap_r = true;
                    }
                }
            } else {
                self.double_tap = Some((*client_instant, tap_u, tap_d, tap_l, tap_r));
            }

            info!(
                "tap_u: {}, tap_d: {}, tap_l: {}, tap_r: {}",
                tap_u, tap_d, tap_l, tap_r
            );
            let mut did_tap = false;
            match (tap_u, tap_d, tap_l, tap_r) {
                (true, false, false, true) => {
                    next_command.set_look(Direction::Northeast);
                    did_tap = true;
                }
                (true, false, true, false) => {
                    next_command.set_look(Direction::Northwest);
                    did_tap = true;
                }
                (false, true, true, false) => {
                    next_command.set_look(Direction::Southwest);
                    did_tap = true;
                }
                (false, true, false, true) => {
                    next_command.set_look(Direction::Southeast);
                    did_tap = true;
                }
                _ => {}
            }
            if !did_tap {
                if self.sequential_tap_instant.is_some() {
                    match (tap_u, tap_d, tap_l, tap_r) {
                        (false, false, false, true) => {
                            // towards east
                            let nextdir = match lookdir {
                                Direction::East
                                | Direction::West
                                | Direction::Northeast
                                | Direction::Southeast => Direction::East,
                                Direction::North => Direction::Northeast,
                                Direction::South => Direction::Southeast,
                                Direction::Northwest => Direction::North,
                                Direction::Southwest => Direction::South,
                            };
                            next_command.set_look(nextdir);
                        }
                        (true, false, false, false) => {
                            // towards north
                            let nextdir = match lookdir {
                                Direction::North
                                | Direction::South
                                | Direction::Northeast
                                | Direction::Northwest => Direction::North,
                                Direction::East => Direction::Northeast,
                                Direction::West => Direction::Northwest,
                                Direction::Southeast => Direction::East,
                                Direction::Southwest => Direction::West,
                            };
                            next_command.set_look(nextdir);
                        }
                        (false, false, true, false) => {
                            // towards west
                            let nextdir = match lookdir {
                                Direction::East
                                | Direction::West
                                | Direction::Northwest
                                | Direction::Southwest => Direction::West,
                                Direction::North => Direction::Northwest,
                                Direction::South => Direction::Southwest,
                                Direction::Northeast => Direction::North,
                                Direction::Southeast => Direction::South,
                            };
                            next_command.set_look(nextdir);
                        }
                        (false, true, false, false) => {
                            // towards south
                            let nextdir = match lookdir {
                                Direction::North
                                | Direction::South
                                | Direction::Southeast
                                | Direction::Southwest => Direction::South,
                                Direction::East => Direction::Southeast,
                                Direction::West => Direction::Southwest,
                                Direction::Northeast => Direction::East,
                                Direction::Northwest => Direction::West,
                            };
                            next_command.set_look(nextdir);
                        }
                        _ => {}
                    }
                } else {
                    match (tap_u, tap_d, tap_l, tap_r) {
                        (false, false, false, true) => next_command.set_look(Direction::East),
                        (true, false, false, false) => next_command.set_look(Direction::North),
                        (false, false, true, false) => next_command.set_look(Direction::West),
                        (false, true, false, false) => next_command.set_look(Direction::South),
                        _ => {}
                    }
                }
            }

            self.sequential_tap_instant = Some(*client_instant);
        }
    }

    fn handle_holds(&mut self) {
        let next_command = self.next_command.as_mut().unwrap();

        let mut short_u = false;
        let mut short_d = false;
        let mut short_l = false;
        let mut short_r = false;

        let mut long_u = false;
        let mut long_d = false;
        let mut long_l = false;
        let mut long_r = false;

        if !long_u {
            if let Some((_, duration)) = self.pressed_keys.get(&Key::W) {
                short_u = true;
                if *duration >= MAX_TAP_DURATION {
                    long_u = true;
                }
            }
        }
        if !long_d {
            if let Some((_, duration)) = self.pressed_keys.get(&Key::S) {
                short_d = true;
                if *duration >= MAX_TAP_DURATION {
                    long_d = true;
                }
            }
        }
        if !long_l {
            if let Some((_, duration)) = self.pressed_keys.get(&Key::A) {
                short_l = true;
                if *duration >= MAX_TAP_DURATION {
                    long_l = true;
                }
            }
        }
        if !long_r {
            if let Some((_, duration)) = self.pressed_keys.get(&Key::D) {
                short_r = true;
                if *duration >= MAX_TAP_DURATION {
                    long_r = true;
                }
            }
        }

        if short_u && short_d {
            short_u = false;
            short_d = false;
        }
        if short_l && short_r {
            short_l = false;
            short_r = false;
        }
        if long_u && long_d {
            long_u = false;
            long_d = false;
        }
        if long_l && long_r {
            long_l = false;
            long_r = false;
        }

        // Movement
        if long_u || long_d || long_l || long_r {
            match (short_u, short_d, short_l, short_r) {
                (false, false, false, true) => next_command.set_move(Direction::East),
                (true, false, false, true) => next_command.set_move(Direction::Northeast),
                (true, false, false, false) => next_command.set_move(Direction::North),
                (true, false, true, false) => next_command.set_move(Direction::Northwest),
                (false, false, true, false) => next_command.set_move(Direction::West),
                (false, true, true, false) => next_command.set_move(Direction::Southwest),
                (false, true, false, false) => next_command.set_move(Direction::South),
                (false, true, false, true) => next_command.set_move(Direction::Southeast),
                (false, false, false, false) => {}
                _ => panic!("Invalid move command"),
            }
        }
    }

    pub fn pop_outgoing_command(&mut self) -> Option<PlayerCommands> {
        self.next_command.take()
    }

    pub fn save_to_command_history(
        &mut self,
        client_tick: Tick,
        command_opt: Option<PlayerCommands>,
    ) {
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
        self.command_history
            .insert(client_tick, command_opt.clone());
    }

    pub fn pop_command_replays(
        &mut self,
        server_tick: Tick,
    ) -> Vec<(Tick, Option<PlayerCommands>)> {
        self.command_history.replays(&server_tick)
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
