use bevy_state::state::States;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum AppState {
    Loading,
    MainMenu,
    InGame,
}
