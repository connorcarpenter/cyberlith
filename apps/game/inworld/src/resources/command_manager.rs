use bevy_ecs::system::Resource;

use game_engine::world;
use game_engine::world::messages::{CommandReadState, KeyCommand};

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
    pub fn recv_command(&mut self, key_command_opt: Option<KeyCommand>) {
        self.internal.recv_command(key_command_opt);
    }

    pub fn take_commands(&mut self) -> Vec<CommandReadState> {
        self.internal.take_commands()
    }
}