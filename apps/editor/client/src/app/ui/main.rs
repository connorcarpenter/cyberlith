use bevy_ecs::system::Res;

use render_egui::{egui, EguiContext};

use crate::app::ui::{left_panel, right_panel, top_bar, center_panel};

pub fn main(
    context: Res<EguiContext>,
) {
    let context = context.inner();
    top_bar(context);
    left_panel(context);
    right_panel(context);
    center_panel(context);
}