use bevy_ecs::{entity::Entity, world::World};
use render_egui::egui::{Align, Layout, Ui};

use vortex_proto::components::{EntryKind, FileSystemEntry};

use crate::app::{
    components::file_system::{FileSystemParent, FileSystemUiState},
    resources::file_manager::FileManager,
    ui::widgets::FileTreeRowUiWidget,
};

pub struct FileTreeUiWidget;

impl FileTreeUiWidget {
    pub fn render_root(ui: &mut Ui, world: &mut World) {
        let root_entity = world
            .get_resource::<FileManager>()
            .unwrap()
            .project_root_entity;
        let entry = world.entity(root_entity).get::<FileSystemEntry>().unwrap();
        let name = (*entry.name).clone();

        ui.with_layout(Layout::top_down_justified(Align::LEFT), |ui| {
            Self::render(ui, world, &root_entity, "", &name, 0);
        });
    }

    fn render(
        ui: &mut Ui,
        world: &mut World,
        entity: &Entity,
        path: &str,
        name: &str,
        depth: usize,
    ) {
        let is_directory =
            *(world.entity(*entity).get::<FileSystemEntry>().unwrap().kind) == EntryKind::Directory;

        if is_directory {
            FileTreeRowUiWidget::render_row(ui, world, entity, path, name, depth, true);
            let Some(ui_state) = world
                .entity(*entity)
                .get::<FileSystemUiState>() else {
                return;
            };
            if ui_state.opened {
                Self::render_children(ui, world, entity, path, name, depth);
            }
        } else {
            FileTreeRowUiWidget::render_row(ui, world, entity, path, name, depth, false);
        }
    }

    fn render_children(
        ui: &mut Ui,
        world: &mut World,
        entity: &Entity,
        path: &str,
        name: &str,
        depth: usize,
    ) {
        let separator = if path.len() > 0 { ":" } else { "" };
        let full_path = format!("{}{}{}", path, separator, name);

        let parent = world.entity(*entity).get::<FileSystemParent>().unwrap();

        for child_entity in parent.get_children() {
            if let Some(entity_ref) = world.get_entity(child_entity) {
                if let Some(entry) = entity_ref.get::<FileSystemEntry>() {
                    let child_name = entry.name.clone();
                    Self::render(ui, world, &child_entity, &full_path, &child_name, depth + 1);
                }
            }
        }
    }
}
