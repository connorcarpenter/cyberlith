use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{
    event::{EventReader, EventWriter},
    prelude::World as BevyWorld,
};

use naia_bevy_client::{
    component_events::{
        component_events_startup, get_component_events, AppRegisterComponentEvents,
        InsertComponentEvent, RemoveComponentEvent, UpdateComponentEvent,
    },
    NaiaClientError,
};

use kernel::AppExitAction;
use logging::{info, warn};

use session_server_naia_proto::components::{
    ChatMessage, ChatMessageGlobal, ChatMessageLocal, Lobby, LobbyMember, Selfhood, SelfhoodUser,
    User,
};

use crate::{
    connection_manager::ConnectionManager,
    client_markers::Session,
    session::{SessionDespawnEntityEvent, SessionErrorEvent, SessionSpawnEntityEvent},
};

pub type SessionInsertComponentEvent<C> = InsertComponentEvent<Session, C>;
pub type SessionUpdateComponentEvent<C> = UpdateComponentEvent<Session, C>;
pub type SessionRemoveComponentEvent<C> = RemoveComponentEvent<Session, C>;

pub struct SessionEventsPlugin;

impl Plugin for SessionEventsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, ConnectionManager::handle_session_connect_events)
            .add_systems(Update, ConnectionManager::handle_session_disconnect_events)
            .add_systems(Update, ConnectionManager::handle_session_reject_events)
            .add_systems(Update, ConnectionManager::handle_session_message_events)
            .add_systems(Update, ConnectionManager::handle_session_request_events)
            .add_systems(Update, error_events)
            .add_systems(Update, spawn_entity_events)
            .add_event::<SessionSpawnEntityEvent>()
            .add_systems(Update, despawn_entity_events)
            .add_event::<SessionDespawnEntityEvent>()
            // component events
            .add_systems(Startup, component_events_startup::<Session>)
            .add_systems(Update, component_events_update)
            .add_component_events::<Session, User>()
            .add_component_events::<Session, ChatMessage>()
            .add_component_events::<Session, ChatMessageGlobal>()
            .add_component_events::<Session, ChatMessageLocal>()
            .add_component_events::<Session, Lobby>()
            .add_component_events::<Session, LobbyMember>()
            .add_component_events::<Session, Selfhood>()
            .add_component_events::<Session, SelfhoodUser>();
    }
}

// used as a system
fn component_events_update(world: &mut BevyWorld) {
    for events in get_component_events::<Session>(world) {
        events.process::<User>(world);
        events.process::<ChatMessage>(world);
        events.process::<ChatMessageGlobal>(world);
        events.process::<ChatMessageLocal>(world);
        events.process::<Lobby>(world);
        events.process::<LobbyMember>(world);
        events.process::<Selfhood>(world);
        events.process::<SelfhoodUser>(world);
    }
}

// used as a system
fn spawn_entity_events(mut event_reader: EventReader<SessionSpawnEntityEvent>) {
    for _event in event_reader.read() {
        // info!("spawned entity");
    }
}

// used as a system
fn despawn_entity_events(mut event_reader: EventReader<SessionDespawnEntityEvent>) {
    for _event in event_reader.read() {
        // info!("despawned entity");
    }
}

// used as a system
fn error_events(
    mut event_reader: EventReader<SessionErrorEvent>,
    mut app_exit_action_writer: EventWriter<AppExitAction>,
) {
    for event in event_reader.read() {
        let error = &event.err;
        match error {
            NaiaClientError::IdError(status_code) => {
                match status_code {
                    409 => {
                        // conflict, represents attempted simultaneous connection
                        warn!("SessionErrorEvent::IdError(CONFLICT!)");

                        // redirect to launcher
                        redirect_to_launcher_app(&mut app_exit_action_writer)
                    }
                    _ => {
                        warn!(
                            "SessionErrorEvent::IdError, with unhandled status code: {:?}",
                            status_code
                        );
                    }
                }
            }
            error => {
                warn!("SessionErrorEvent: {:?}", error);
            }
        }
    }
}

fn redirect_to_launcher_app(app_exit_action_writer: &mut EventWriter<AppExitAction>) {
    info!("redirecting to launcher app");
    app_exit_action_writer.send(AppExitAction::go_to("launcher"));
}
