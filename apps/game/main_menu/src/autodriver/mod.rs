use bevy_app::{App, Plugin, Startup, Update};
use bevy_ecs::{system::Query, event::{EventWriter}, system::{Res, ResMut, Resource}};

use game_engine::{session::components::User, logging::{info}, ui::UiManager};

use crate::{resources::user_manager::UserManager, ui::{UiCatalog, UiKey, events::{HostMatchButtonClickedEvent, JoinMatchButtonClickedEvent}}};

pub(crate) struct AutodriverPlugin;

impl Plugin for AutodriverPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<AutodriverState>()
            .add_systems(Startup, startup)
            .add_systems(Update, update);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MenuState {
    NoName,
    LandingHost,
    LandingOther,
    InHostGame,
    InJoinGame,
    InLobby,
}

#[derive(Resource)]
struct AutodriverState {
    menu_state: MenuState,
}

impl Default for AutodriverState {
    fn default() -> Self {
        Self {
            menu_state: MenuState::NoName,
        }
    }
}

fn startup(
    _state: Res<AutodriverState>,
) {
    info!("Launcher::Autodriver::startup()");
}

fn update(
    mut state: ResMut<AutodriverState>,
    user_manager: Res<UserManager>,
    ui_manager: Res<UiManager>,
    ui_catalog: Res<UiCatalog>,
    user_q: Query<&User>,
    mut host_match_btn_wrtr: EventWriter<HostMatchButtonClickedEvent>,
    mut join_match_btn_wrtr: EventWriter<JoinMatchButtonClickedEvent>,
) {
    let Some(active_ui_handle) = ui_manager.active_ui() else {
        return;
    };
    if ui_catalog.get_ui_key(&active_ui_handle) != UiKey::MainMenu {
        return;
    }

    match state.menu_state {
        MenuState::NoName => {
            if let Some(self_user_entity) = user_manager.get_self_user_entity() {
                if let Ok(user) = user_q.get(self_user_entity) {

                    if user.name.eq_ignore_ascii_case("connor") {
                        state.menu_state = MenuState::LandingHost;
                    } else {
                        state.menu_state = MenuState::LandingOther;
                    }
                }
            }
        }
        MenuState::LandingHost => {

            if let Some(current_ui_handle) =
                ui_manager.get_ui_container_contents(&active_ui_handle, "center_container")
            {
                let ui_key = ui_catalog.get_ui_key(&current_ui_handle);
                if UiKey::HostMatch == ui_key {
                    state.menu_state = MenuState::InHostGame;
                } else {
                    host_match_btn_wrtr.send(HostMatchButtonClickedEvent);
                }
            }

        }
        MenuState::LandingOther => {

            if let Some(current_ui_handle) =
                ui_manager.get_ui_container_contents(&active_ui_handle, "center_container")
            {
                let ui_key = ui_catalog.get_ui_key(&current_ui_handle);
                if UiKey::JoinMatch == ui_key {
                    state.menu_state = MenuState::InJoinGame;
                } else {
                    join_match_btn_wrtr.send(JoinMatchButtonClickedEvent);
                }
            }
        }
        MenuState::InHostGame => {}
        MenuState::InJoinGame => {}
        MenuState::InLobby => {}
    }
}