use std::collections::BTreeMap;

use naia_bevy_shared::Tick;

use crate::messages::PlayerCommand;

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