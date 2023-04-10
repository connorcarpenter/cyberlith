use std::borrow::BorrowMut;

use bevy_ecs::{
    system::{Res, ResMut},
    world::World,
};

use render_egui::{egui, EguiContext};

use crate::app::ui::{center_panel, left_panel, right_panel, top_bar, UiState};

pub fn main(world: &mut World) {
    let context = world.get_resource::<EguiContext>().unwrap().inner().clone();
    top_bar(&context, world);
    left_panel(&context, world);
    right_panel(&context, world);
    center_panel(&context, world);
}
