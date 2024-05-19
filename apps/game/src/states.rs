use bevy_ecs::schedule::States;

#[derive(States, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub(crate) enum AppState {
    Loading,
    MainMenu,
    InGame,
}
