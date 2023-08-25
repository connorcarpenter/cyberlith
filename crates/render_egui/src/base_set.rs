
use bevy_ecs::schedule::ScheduleLabel;

#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct EguiPreUpdate;

#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct EguiPostUpdate;

#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct EguiSync;

#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct EguiDraw;