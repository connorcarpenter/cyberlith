use std::collections::{BTreeMap, btree_map::{IntoIter, Iter}};

use naia_bevy_shared::Tick;

use logging::{info, warn};

use crate::{types::Direction, messages::PlayerCommand};

pub struct CommandTimeline {
    tick: Tick,
    inner_opt: Option<CommandTimelineInner>,
}

impl CommandTimeline {
    pub fn new(tick: Tick) -> Self {
        Self {
            tick,
            inner_opt: None,
        }
    }

    pub fn recv_stream(&mut self, command: PlayerCommand, stream_output: (Option<u16>, Vec<(bool, u8)>) ) {
        if self.inner_opt.is_none() {
            self.inner_opt = Some(CommandTimelineInner::new());
        }
        let inner = self.inner_opt.as_mut().unwrap();
        inner.recv_stream(command, stream_output);
    }

    pub fn get_movement_vector(&self, dx: i8, dy: i8) -> (i8, i8) {
        if self.inner_opt.is_none() {
            return (dx, dy);
        }
        let inner = self.inner_opt.as_ref().unwrap();
        inner.get_movement_vector(dx, dy)
    }
}

struct CommandTimelineInner {
    events: BTreeMap<u8, CommandTimelineEvents>,
}

impl CommandTimelineInner {
    fn new() -> Self {
        Self {
            events: BTreeMap::new(),
        }
    }

    fn recv_stream(&mut self, command: PlayerCommand, stream_output: (Option<u16>, Vec<(bool, u8)>) ) {
        info!("recv_stream: {:?}, {:?}", command, stream_output);

        let (start_pressed, durations) = stream_output;
        let mut t = 0;
        for (pressed, duration) in durations {
            if t == 0 {
                if pressed {
                    if start_pressed.is_none() {
                        panic!("start_pressed != first pressed");
                    }
                    self.insert_event(t, command, CommandTimelineEvent::PressStart(start_pressed));
                } else {
                    if start_pressed.is_some() {
                        panic!("start_pressed != first pressed");
                    }
                }
            } else {
                if pressed {
                    self.insert_event(t, command, CommandTimelineEvent::PressStart(None));
                }
            }

            t += duration;
            self.insert_event(t, command, CommandTimelineEvent::PressEnd);
        }
    }

    fn insert_event(&mut self, t: u8, command: PlayerCommand, event: CommandTimelineEvent) {
        if !self.events.contains_key(&t) {
            self.events.insert(t, CommandTimelineEvents::new());
        }
        let events = self.events.get_mut(&t).unwrap();
        events.list.push((command, event));
    }

    fn get_movement_vector(&self, dx: i8, dy: i8) -> (i8, i8) {

        let mut ee = 0;
        let mut ne = 0;
        let mut nn = 0;
        let mut nw = 0;
        let mut ww = 0;
        let mut sw = 0;
        let mut ss = 0;
        let mut se = 0;

        match (dx, dy) {
            // None
            (0, 0) => {},
            // East
            (1, 0) => ee = 150,
            // Northeast
            (1, -1) => ne = 150,
            // North
            (0, -1) => nn = 150,
            // Northwest
            (-1, -1) => nw = 150,
            // West
            (-1, 0) => ww = 150,
            // Southwest
            (-1, 1) => sw = 150,
            // South
            (0, 1) => ss = 150,
            // Southeast
            (1, 1) => se = 150,
            _ => {
                panic!("Invalid movement vector: ({}, {})", dx, dy);
            }
        }

        let mut l = false;
        let mut r = false;
        let mut u = false;
        let mut d = false;

        let mut lsp = 0;
        let mut rsp = 0;
        let mut usp = 0;
        let mut dsp = 0;

        let mut last_t = 0;

        for (t, events) in self.events.iter() {

            let t = *t;
            let since_last = (t - last_t) as u16;
            last_t = t;

            if since_last != 0 {
                let mut cxsp = 0;
                let mut cysp = 0;
                let mut cx = 0;
                let mut cy = 0;
                if l != r {
                    if l { cx = -1; cxsp = lsp; lsp = 0; }
                    if r { cx = 1; cxsp = rsp; rsp = 0; }
                }
                if u != d {
                    if u { cy = -1; cysp = usp; usp = 0; }
                    if d { cy = 1; cysp = dsp; dsp = 0; }
                }

                match (cx, cy) {
                    // None
                    (0, 0) => {},
                    // East
                    (1, 0) => ee += cxsp + since_last,
                    // Northeast
                    (1, -1) => ne += cxsp.min(cysp) + since_last,
                    // North
                    (0, -1) => nn += cysp + since_last,
                    // Northwest
                    (-1, -1) => nw += cxsp.min(cysp) + since_last,
                    // West
                    (-1, 0) => ww += cxsp + since_last,
                    // Southwest
                    (-1, 1) => sw += cxsp.min(cysp) + since_last,
                    // South
                    (0, 1) => ss += cysp + since_last,
                    // Southeast
                    (1, 1) => se += cxsp.min(cysp) + since_last,
                    _ => {
                        panic!("Invalid movement vector: ({}, {})", cx, cy);
                    }
                }

                // decay
                let decay_time = since_last * 2;
                if !(cx == 1 && cy == 0) { if ee > decay_time { ee -= decay_time; } else { ee = 0; } }
                if !(cx == 1 && cy == -1) { if ne > decay_time { ne -= decay_time; } else { ne = 0; } }
                if !(cx == 0 && cy == -1) { if nn > decay_time { nn -= decay_time; } else { nn = 0; } }
                if !(cx == -1 && cy == -1) { if nw > decay_time { nw -= decay_time; } else { nw = 0; } }
                if !(cx == -1 && cy == 0) { if ww > decay_time { ww -= decay_time; } else { ww = 0; } }
                if !(cx == -1 && cy == 1) { if sw > decay_time { sw -= decay_time; } else { sw = 0; } }
                if !(cx == 0 && cy == 1) { if ss > decay_time { ss -= decay_time; } else { ss = 0; } }
                if !(cx == 1 && cy == 1) { if se > decay_time { se -= decay_time; } else { se = 0; } }
            }

            for (command, event) in events.list.iter() {
                match command {
                    PlayerCommand::Up => match event {
                        CommandTimelineEvent::PressStart(start_pressed_opt) => {
                            if let Some(start_pressed) = start_pressed_opt {
                                usp = *start_pressed;
                            }
                            u = true;
                        }
                        CommandTimelineEvent::PressEnd => u = false,
                    }

                    PlayerCommand::Down => match event {
                        CommandTimelineEvent::PressStart(start_pressed_opt) => {
                            if let Some(start_pressed) = start_pressed_opt {
                                dsp = *start_pressed;
                            }
                            d = true;
                        }
                        CommandTimelineEvent::PressEnd => d = false,
                    }
                    PlayerCommand::Left => match event {
                        CommandTimelineEvent::PressStart(start_pressed_opt) => {
                            if let Some(start_pressed) = start_pressed_opt {
                                lsp = *start_pressed;
                            }
                            l = true;
                        }
                        CommandTimelineEvent::PressEnd => l = false,
                    }
                    PlayerCommand::Right => match event {
                        CommandTimelineEvent::PressStart(start_pressed_opt) => {
                            if let Some(start_pressed) = start_pressed_opt {
                                rsp = *start_pressed;
                            }
                            r = true;
                        }
                        CommandTimelineEvent::PressEnd => r = false,
                    }
                }
            }
        }

        let mut winner = (0, 0, 0);
        if ee > winner.0 { winner = (ee, 1, 0); }
        if ne > winner.0 { winner = (ne, 1, -1); }
        if nn > winner.0 { winner = (nn, 0, -1); }
        if nw > winner.0 { winner = (nw, -1, -1); }
        if ww > winner.0 { winner = (ww, -1, 0); }
        if sw > winner.0 { winner = (sw, -1, 1); }
        if ss > winner.0 { winner = (ss, 0, 1); }
        if se > winner.0 { winner = (se, 1, 1); }

        let (held_duration, winner_x, winner_y) = winner;

        if held_duration < 150 {
            // not enough to initiate a tile move,
            // but later will be enough to initiate a looking direction change
            return (0, 0);
        }

        return (winner_x, winner_y);
    }
}

struct CommandTimelineEvents {
    list: Vec<(PlayerCommand, CommandTimelineEvent)>
}

impl CommandTimelineEvents {
    fn new() -> Self {
        Self {
            list: Vec::new(),
        }
    }
}

enum CommandTimelineEvent {
    // (prev duration)
    PressStart(Option<u16>),
    //
    PressEnd,
}