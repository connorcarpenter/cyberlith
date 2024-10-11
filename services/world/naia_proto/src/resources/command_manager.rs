use crate::messages::{CommandReadState, KeyCommand};

// Intended to be encapsulated within a Client or Server specific Resource!
pub struct CommandManager {
    commands: Vec<CommandReadState>
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            commands: Vec::new()
        }
    }

    pub fn recv_command(&mut self, key_command_opt: Option<KeyCommand>) {
        if let Some(key_command) = key_command_opt {
            self.commands.push(key_command.get_read_state());
        } else {
            self.commands.push(KeyCommand::new().get_read_state());
        }
    }

    pub fn take_commands(&mut self) -> Vec<CommandReadState> {
        std::mem::take(&mut self.commands)
    }
}