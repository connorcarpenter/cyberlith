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
        let mut kx: i8 = dx; // -1, 0, 1
        let mut ky: i8 = dy; // -1, 0, 1
        let mut mx: u16 = if dx != 0 { 150 } else { 0 }; // time in ms
        let mut my: u16 = if dy != 0 { 150 } else { 0 }; // time in ms

        let mut l = false;
        let mut r = false;
        let mut u = false;
        let mut d = false;

        let mut ne = dx == 1 && dy == -1;
        let mut nw = dx == -1 && dy == -1;
        let mut se = dx == 1 && dy == 1;
        let mut sw = dx == -1 && dy == 1;

        let mut lsp = 0;
        let mut rsp = 0;
        let mut usp = 0;
        let mut dsp = 0;

        let mut last_t = 0;

        for (t, events) in self.events.iter() {

            let t = *t;
            let since_last = t - last_t;
            last_t = t;

            if since_last != 0 {
                if u && r {
                    ne = true;
                }
                if u && l {
                    nw = true;
                }
                if d && r {
                    se = true;
                }
                if d && l {
                    sw = true;
                }

                if l != r {
                    if l {
                        mx += lsp;
                        lsp = 0;
                        if kx == 1 {
                            mx = 0;
                        }
                        if kx != -1 {
                            kx = -1;
                            if ky == 1 && !sw {
                                ky = 0;
                            }
                            if ky == -1 && !nw {
                                ky = 0;
                            }
                        }
                    }
                    if r {
                        mx += rsp;
                        rsp = 0;
                        if kx == -1 {
                            mx = 0;
                        }
                        if kx != 1 {
                            kx = 1;
                            if ky == 1 && !se {
                                ky = 0;
                            }
                            if ky == -1 && !ne {
                                ky = 0;
                            }
                        }
                    }
                    mx += since_last as u16;
                } else {
                    if l && r {
                        kx = 0;
                        mx = 0;
                        lsp = 0;
                        rsp = 0;
                    } else {
                        if mx > 0 {
                            if mx > since_last as u16 {
                                mx = (mx - since_last as u16);
                            } else {
                                mx = 0;
                                kx = 0;
                            }
                        }
                    }
                }

                if u != d {
                    if u {
                        my += usp;
                        usp = 0;
                        if ky == 1 {
                            my = 0;
                        }
                        if ky != -1 {
                            ky = -1;
                            if kx == 1 && !ne {
                                kx = 0;
                            }
                            if kx == -1 && !nw {
                                kx = 0;
                            }
                        }
                    }
                    if d {
                        my += dsp;
                        dsp = 0;
                        if ky == -1 {
                            my = 0;
                        }
                        if ky != 1 {
                            ky = 1;
                            if kx == 1 && !se {
                                kx = 0;
                            }
                            if kx == -1 && !sw {
                                kx = 0;
                            }
                        }
                    }
                    my += since_last as u16;
                } else {
                    if u && d {
                        ky = 0;
                        my = 0;
                        usp = 0;
                        dsp = 0;
                    } else {
                        if my > 0 {
                            if my > since_last as u16 {
                                my = (my - since_last as u16);
                            } else {
                                my = 0;
                                ky = 0;
                            }
                        }
                    }
                }
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

        if kx == 0 && ky == 0 {
            // not moving
            return (0, 0);
        }
        if mx < 150 && my < 150 {
            // not enough to initiate a tile move,
            // but later will be enough to initiate a looking direction change
            return (0, 0);
        }

        return (kx, ky);
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