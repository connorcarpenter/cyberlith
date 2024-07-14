mod chat_message;
mod lobby;
mod user;

pub use chat_message::*;
pub use lobby::*;
pub use user::*;

//

use bevy_app::{App, Plugin};

pub struct SessionComponentEventsPlugin;

impl Plugin for SessionComponentEventsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ChatMessageComponentEventsPlugin)
            .add_plugins(LobbyComponentEventsPlugin)
            .add_plugins(UserComponentEventsPlugin);
    }
}
