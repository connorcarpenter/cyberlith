use bevy_ecs::prelude::{Entity, World};

use vortex_proto::components::EntryKind;

use crate::app::resources::{
    action::{create_file, delete_file, rename_file, select_file, Action, ActionStack},
    file_tree::FileTree,
};

#[derive(Clone)]
pub enum FileAction {
    // A list of File Row entities to select
    SelectFile(Vec<Entity>),
    // The directory entity to add the new Entry to, the name of the new Entry, it's Kind, an older Entity it was associated with if necessary, and a list of child Entries to create
    CreateFile(
        Option<Entity>,
        String,
        EntryKind,
        Option<Entity>,
        Option<Vec<FileTree>>,
    ),
    // The File Row entity to delete, and a list of entities to select after deleted
    DeleteFile(Entity, Option<Vec<Entity>>),
    // The File Row entity to rename, and the new name
    RenameFile(Entity, String),
}

pub enum FileActionType {
    SelectFile,
    CreateFile,
    DeleteFile,
    RenameFile,
}

impl FileAction {

    pub(crate) fn get_type(&self) -> FileActionType {
        match self {
            Self::SelectFile(_) => FileActionType::SelectFile,
            Self::CreateFile(_, _, _, _, _) => FileActionType::CreateFile,
            Self::DeleteFile(_, _) => FileActionType::DeleteFile,
            Self::RenameFile(_, _) => FileActionType::RenameFile,
        }
    }

    pub(crate) fn migrate_file_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        match self {
            FileAction::SelectFile(entities) => {
                for entity in entities {
                    if *entity == old_entity {
                        *entity = new_entity;
                    }
                }
            }
            FileAction::CreateFile(entity_opt, _, _, entity_opt_2, _) => {
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
            FileAction::DeleteFile(entity, entities_opt) => {
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
            FileAction::RenameFile(entity, _) => {
                if *entity == old_entity {
                    *entity = new_entity;
                }
            }
        }
    }
}

impl Action for FileAction {
    fn execute(
        self,
        world: &mut World,
        entity_opt: Option<&Entity>,
        action_stack: &mut ActionStack<Self>,
    ) -> Vec<Self> {
        match self {
            Self::SelectFile(file_entities) => select_file::execute(world, file_entities),
            Self::CreateFile(
                parent_entity_opt,
                new_file_name,
                entry_kind,
                old_entity_opt,
                entry_contents_opt,
            ) => {
                let project_root_entity = *(entity_opt.unwrap());
                create_file::execute(
                    world,
                    action_stack,
                    project_root_entity,
                    parent_entity_opt,
                    new_file_name,
                    entry_kind,
                    old_entity_opt,
                    entry_contents_opt,
                )
            }
            Self::DeleteFile(file_entity, files_to_select_opt) => {
                let project_root_entity = *(entity_opt.unwrap());
                delete_file::execute(world, project_root_entity, file_entity, files_to_select_opt)
            }
            Self::RenameFile(file_entity, new_name) => {
                rename_file::execute(world, file_entity, new_name)
            }
        }
    }

    fn entity_update_auth_status_impl(
        buffered_check: &mut bool,
        action_opt: Option<&Self>,
        entity: &Entity,
    ) {
        match action_opt {
            Some(Self::SelectFile(file_entities)) => {
                if file_entities.contains(entity) {
                    *buffered_check = true;
                }
            }
            _ => {}
        }
    }

    fn enable_top_impl(world: &mut World, last_action: Option<&Self>, enabled: &mut bool) {
        match last_action {
            Some(Self::SelectFile(entities)) => {
                *enabled = ActionStack::<FileAction>::should_be_enabled(world, entities);
            }
            _ => {
                *enabled = true;
            }
        }
    }
}
