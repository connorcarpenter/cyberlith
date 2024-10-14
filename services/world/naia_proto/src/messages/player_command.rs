use std::collections::HashMap;

use naia_bevy_shared::{Message, Serde, Tick, UnsignedInteger};

use logging::{info, warn};

#[derive(Serde, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum PlayerCommand {
    Up,
    Down,
    Left,
    Right,
}

impl PlayerCommand {
    pub fn as_str(&self) -> &str {
        match self {
            PlayerCommand::Up => "Forward",
            PlayerCommand::Down => "Backward",
            PlayerCommand::Left => "Left",
            PlayerCommand::Right => "Right",
        }
    }
}

#[derive(Message)]
pub struct PlayerCommands {
    map: HashMap<PlayerCommand, PlayerCommandStream>,
}

impl PlayerCommands {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn get(&self, command: &PlayerCommand) -> Option<&PlayerCommandStream> {
        self.map.get(command)
    }

    pub fn set(&mut self, command: PlayerCommand, stream: PlayerCommandStream) {
        self.map.insert(command, stream);
    }

    pub fn log(&self, tick: Tick) {
        info!("Processing PlayerCommands for Tick({:?})", tick);
        for (key, value) in &self.map {
            value.log(key);
        }
        info!("---");
    }
}

#[derive(Serde, Clone, PartialEq)]
pub struct PlayerCommandStream {
    start_pressed: bool,
    durations: Vec<UnsignedInteger<6>>,
}

impl PlayerCommandStream {
    pub fn new(start_pressed: bool) -> Self {
        Self {
            start_pressed,
            durations: Vec::new(),
        }
    }

    pub fn add_duration(&mut self, mut duration: u8) {
        if duration > 63 {
            warn!("Attempted to add duration > 63 millis! ({}ms)", duration);
            duration = 63;
        }
        self.durations.push(UnsignedInteger::new(duration));
    }

    pub fn start_pressed(&self) -> bool {
        self.start_pressed
    }

    pub fn durations(&self) -> &Vec<UnsignedInteger<6>> {
        &self.durations
    }

    fn log(&self, key: &PlayerCommand) {
        let mut pressed = self.start_pressed;
        for duration in &self.durations {
            if pressed {
                info!("'{}' pressed for {}ms", key.as_str(), duration.get());
            } else {
                info!("'{}' released for {}ms", key.as_str(), duration.get());
            }
            pressed = !pressed;
        }

        if pressed {
            info!("'{}' pressed for remainder of tick", key.as_str());
        } else {
            info!("'{}' released  for remainder of tick", key.as_str());
        }
    }
}