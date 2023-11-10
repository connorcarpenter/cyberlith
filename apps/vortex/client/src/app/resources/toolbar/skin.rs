
use bevy_ecs::{world::{Mut, World}, entity::Entity};

use render_egui::egui::Ui;

use crate::app::resources::{input::InputManager, skin_manager::SkinManager, tab_manager::TabManager};

pub struct SkinToolbar;

impl SkinToolbar {
    pub fn render(world: &mut World, ui: &mut Ui, current_file_entity: &Entity) {
        // Toolbar
        let mut some_action = None;
        world.resource_scope(|world, mut skin_manager: Mut<SkinManager>| {
            some_action =
                skin_manager.render_sidebar(ui, world, current_file_entity);
        });
        if let Some(skin_action) = some_action {
            world.resource_scope(|world, mut tab_manager: Mut<TabManager>| {
                world.resource_scope(
                    |world, mut input_manager: Mut<InputManager>| {
                        tab_manager.current_tab_execute_skin_action(
                            world,
                            &mut input_manager,
                            skin_action,
                        );
                    },
                );
            });
        }
    }
}