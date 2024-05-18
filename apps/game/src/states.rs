use bevy_ecs::schedule::States;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum AppState {
    Loading,
    MainMenu,
    InGame,
}