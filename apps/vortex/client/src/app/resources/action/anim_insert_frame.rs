use bevy_ecs::{
    prelude::World,
    system::SystemState,
};
use bevy_log::info;

use crate::app::resources::action::AnimAction;

pub fn execute(world: &mut World, action: AnimAction) -> Vec<AnimAction> {
    let AnimAction::InsertFrame = action else {
        panic!("Expected InsertFrame");
    };

    info!("InsertFrame");

    let mut system_state: SystemState<()> = SystemState::new(world);
    let () = system_state.get_mut(world);

    system_state.apply(world);

    return vec![AnimAction::DeleteFrame];
}
