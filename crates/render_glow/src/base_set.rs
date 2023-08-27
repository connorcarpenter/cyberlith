use bevy_ecs::schedule::ScheduleLabel;

#[derive(Debug, Hash, PartialEq, Eq, Clone, ScheduleLabel)]
pub struct GlowInput;
