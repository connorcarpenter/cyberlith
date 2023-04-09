use std::borrow::BorrowMut;
use bevy_ecs::system::{Res, ResMut};

use render_egui::{egui, EguiContext};

use crate::app::ui::{left_panel, right_panel, top_bar, center_panel, UiState};

pub fn main(
    context: Res<EguiContext>,
    mut state: ResMut<UiState>,
) {
    let ctx = context.inner();
    let state_mut = state.borrow_mut();
    top_bar(ctx, state_mut);
    left_panel(ctx, state_mut);
    right_panel(ctx, state_mut);
    center_panel(ctx, state_mut);
}