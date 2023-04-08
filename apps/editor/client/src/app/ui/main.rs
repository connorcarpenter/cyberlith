use bevy_ecs::system::Res;

use render_egui::{egui, EguiContext};

use crate::app::ui::top_bar::top_bar;

pub fn main(
    context: Res<EguiContext>,
) {
    let context = context.inner();
    top_bar(context);
}