
use bevy_ecs::prelude::{Entity, World};

use vortex_proto::components::EntryKind;

use crate::app::resources::{action_stack::{ActionStack, Action}, file_tree::FileTree, action::{delete_entry, new_entry, rename_entry, select_entries}};

#[derive(Clone)]
pub enum FileAction {
    // A list of File Row entities to select
    SelectEntries(Vec<Entity>),
    // The directory entity to add the new Entry to, the name of the new Entry, it's Kind, an older Entity it was associated with if necessary, and a list of child Entries to create
    NewEntry(
        Option<Entity>,
        String,
        EntryKind,
        Option<Entity>,
        Option<Vec<FileTree>>,
    ),
    // The File Row entity to delete, and a list of entities to select after deleted
    DeleteEntry(Entity, Option<Vec<Entity>>),
    // The File Row entity to rename, and the new name
    RenameEntry(Entity, String),
}

impl FileAction {
    pub(crate) fn migrate_file_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        match self {
            FileAction::SelectEntries(entities) => {
                for entity in entities {
                    if *entity == old_entity {
                        *entity = new_entity;
                    }
                }
            }
            FileAction::NewEntry(entity_opt, _, _, entity_opt_2, _) => {
                if let Some(entity) = entity_opt {
                    if *entity == old_entity {
                        *entity = new_entity;
                    }
                }
                if let Some(entity) = entity_opt_2 {
                    if *entity == old_entity {
                        *entity = new_entity;
                    }
                }
            }
            FileAction::DeleteEntry(entity, entities_opt) => {
                if *entity == old_entity {
                    *entity = new_entity;
                }
                if let Some(entities) = entities_opt {
                    for entity in entities {
                        if *entity == old_entity {
                            *entity = new_entity;
                        }
                    }
                }
            }
            FileAction::RenameEntry(entity, _) => {
                if *entity == old_entity {
                    *entity = new_entity;
                }
            }
        }
    }
}

impl Action for FileAction {
    fn execute(self, world: &mut World, entity_opt: Option<&Entity>, action_stack: &mut ActionStack<Self>) -> Vec<Self> {
        match self {
            Self::SelectEntries(file_entities) => select_entries::execute(world, file_entities),
            Self::NewEntry(
                parent_entity_opt,
                new_file_name,
                entry_kind,
                old_entity_opt,
                entry_contents_opt,
            ) => {
                let project_root_entity = *(entity_opt.unwrap());
                new_entry::execute(
                    world,
                    action_stack,
                    project_root_entity,
                    parent_entity_opt,
                    new_file_name,
                    entry_kind,
                    old_entity_opt,
                    entry_contents_opt,
                )
            },
            Self::DeleteEntry(file_entity, files_to_select_opt) => {
                let project_root_entity = *(entity_opt.unwrap());
                delete_entry::execute(world, project_root_entity, file_entity, files_to_select_opt)
            }
            Self::RenameEntry(file_entity, new_name) => {
                rename_entry::execute(world, file_entity, new_name)
            }
        }
    }

    fn entity_update_auth_status_impl(
        buffered_check: &mut bool,
        action_opt: Option<&Self>,
        entity: &Entity,
    ) {
        match action_opt {
            Some(Self::SelectEntries(file_entities)) => {
                if file_entities.contains(entity) {
                    *buffered_check = true;
                }
            }
            _ => {}
        }
    }

    fn enable_top_impl(world: &mut World, last_action: Option<&Self>, enabled: &mut bool) {
        match last_action {
            Some(Self::SelectEntries(entities)) => {
                *enabled = ActionStack::<FileAction>::should_be_enabled(world, entities);
            }
            _ => {
                *enabled = true;
            }
        }
    }
}