use bevy_ecs::prelude::SystemSet;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
#[system_set(base)]
pub struct EguiDrawSet;
