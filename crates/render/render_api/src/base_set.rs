use bevy_ecs::schedule::ScheduleLabel;

#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct RenderSync;

#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct Draw;

#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct Render;