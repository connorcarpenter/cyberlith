use bevy_ecs::system::Resource;

use game_engine::{world, world::messages::{CommandReadState, KeyCommand}, naia::Tick};
use game_engine::world::resources::KeyEvent;

#[derive(Resource)]
pub struct CommandManager {
    internal: world::resources::CommandManager,
}

impl Default for CommandManager {
    fn default() -> Self {
        Self {
            internal: world::resources::CommandManager::new(),
        }
    }
}

impl CommandManager {
    pub fn recv_command(&mut self, tick: Tick, key_command_opt: Option<KeyCommand>) {
        self.internal.recv_command(tick, key_command_opt);
    }

    pub fn take_commands(&mut self, tick: Tick) -> Vec<KeyEvent> {
        self.internal.take_commands(tick)
    }
}