use bevy_ecs::schedule::ScheduleLabel;

#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct RenderSync;

// app-space, apps should put drawing systems in here that add things to RenderFrame
#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct Draw;

// only renderer should add systems to this!
#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct Render;
