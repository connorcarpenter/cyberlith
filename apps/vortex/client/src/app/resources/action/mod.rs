mod create_edge;
mod create_vertex;
mod delete_edge;
mod delete_entry;
mod delete_face;
mod delete_vertex;
mod move_vertex;
mod new_entry;
mod rename_entry;
mod select_entries;
mod select_shape;

use bevy_ecs::prelude::{Entity, World};

use math::Vec3;

use vortex_proto::components::EntryKind;

use crate::app::{
    components::VertexTypeData,
    resources::{action_stack::ActionStack, file_tree::FileTree, shape_manager::CanvasShape},
};

#[derive(Clone)]
pub enum Action {
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
    // The 2D shape entity to deselect (or None for deselect)
    SelectShape(Option<(Entity, CanvasShape)>),
    // Create Vertex (Vertex-specific data, Position, older vertex 2d entity & 3d entity it was associated with)
    CreateVertex(VertexTypeData, Vec3, Option<(Entity, Entity)>),
    // Delete Vertex (2d vertex entity, optional vertex 2d entity to select after delete)
    DeleteVertex(Entity, Option<(Entity, CanvasShape)>),
    // Move Vertex (2d vertex Entity, Old Position, New Position)
    MoveVertex(Entity, Vec3, Vec3),
    // Create Edge (2d vertex start entity, 2d vertex end entity, 2d shape to select, Option<Vec<(other 2d vertex entity to make a face with, old 2d face entity it was associated with)>>, Option<(older edge 2d entity)>)
    CreateEdge(
        Entity,
        Entity,
        (Entity, CanvasShape),
        Option<Vec<(Entity, Entity, bool)>>,
        Option<Entity>,
    ),
    // Delete Edge (2d edge entity, optional vertex 2d entity to select after delete)
    DeleteEdge(Entity, Option<(Entity, CanvasShape)>),
    // Delete Face (2d face entity)
    DeleteFace(Entity),
}

impl Action {
    pub(crate) fn migrate_file_entities(&mut self, old_entity: Entity, new_entity: Entity) {
        match self {
            Action::SelectEntries(entities) => {
                for entity in entities {
                    if *entity == old_entity {
                        *entity = new_entity;
                    }
                }
            }
            Action::NewEntry(entity_opt, _, _, entity_opt_2, _) => {
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
            Action::DeleteEntry(entity, entities_opt) => {
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
            Action::RenameEntry(entity, _) => {
                if *entity == old_entity {
                    *entity = new_entity;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn migrate_vertex_entities(
        &mut self,
        old_2d_vert_entity: Entity,
        new_2d_vert_entity: Entity,
        old_3d_vert_entity: Entity,
        new_3d_vert_entity: Entity,
    ) {
        match self {
            Action::SelectShape(entity_opt) => match entity_opt {
                Some((entity, CanvasShape::Vertex)) | Some((entity, CanvasShape::RootVertex)) => {
                    if *entity == old_2d_vert_entity {
                        *entity = new_2d_vert_entity;
                    }
                }
                _ => {}
            },
            Action::CreateVertex(vertex_type_data, _, entity_opt) => {
                vertex_type_data.migrate_vertex_entities(
                    old_2d_vert_entity,
                    new_2d_vert_entity,
                    old_3d_vert_entity,
                    new_3d_vert_entity,
                );

                if let Some((other_2d_entity, other_3d_entity)) = entity_opt {
                    if *other_2d_entity == old_2d_vert_entity {
                        *other_2d_entity = new_2d_vert_entity;
                    }
                    if *other_3d_entity == old_3d_vert_entity {
                        *other_3d_entity = new_3d_vert_entity;
                    }
                }
            }
            Action::DeleteVertex(entity, entity_opt) => {
                if *entity == old_2d_vert_entity {
                    *entity = new_2d_vert_entity;
                }
                if let Some((other_entity, _)) = entity_opt {
                    if *other_entity == old_2d_vert_entity {
                        *other_entity = new_2d_vert_entity;
                    }
                }
            }
            Action::MoveVertex(entity, _, _) => {
                if *entity == old_2d_vert_entity {
                    *entity = new_2d_vert_entity;
                }
            }
            Action::CreateEdge(entity_a, entity_b, shape_to_select, face_to_create_opt, _) => {
                if *entity_a == old_2d_vert_entity {
                    *entity_a = new_2d_vert_entity;
                }
                if *entity_b == old_2d_vert_entity {
                    *entity_b = new_2d_vert_entity;
                }
                if let (entity, CanvasShape::Vertex) = shape_to_select {
                    if *entity == old_2d_vert_entity {
                        *entity = new_2d_vert_entity;
                    }
                }
                if let Some(entities) = face_to_create_opt {
                    for (entity, _, _) in entities {
                        if *entity == old_2d_vert_entity {
                            *entity = new_2d_vert_entity;
                        }
                    }
                }
            }
            Action::DeleteEdge(_, Some((entity, _))) => {
                if *entity == old_2d_vert_entity {
                    *entity = new_2d_vert_entity;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn migrate_edge_entities(
        &mut self,
        old_2d_edge_entity: Entity,
        new_2d_edge_entity: Entity,
    ) {
        match self {
            Action::SelectShape(entity_opt) => match entity_opt {
                Some((entity, CanvasShape::Edge)) => {
                    if *entity == old_2d_edge_entity {
                        *entity = new_2d_edge_entity;
                    }
                }
                _ => {}
            },
            Action::CreateVertex(vertex_type_data, _, _) => {
                vertex_type_data.migrate_edge_entities(old_2d_edge_entity, new_2d_edge_entity);
            }
            Action::CreateEdge(_, _, shape_to_select, _, Some(edge_2d_entity)) => {
                if *edge_2d_entity == old_2d_edge_entity {
                    *edge_2d_entity = new_2d_edge_entity;
                }
                if let (entity, CanvasShape::Edge) = shape_to_select {
                    if *entity == old_2d_edge_entity {
                        *entity = new_2d_edge_entity;
                    }
                }
            }
            Action::DeleteEdge(edge_2d_entity, _) => {
                if *edge_2d_entity == old_2d_edge_entity {
                    *edge_2d_entity = new_2d_edge_entity;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn migrate_face_entities(
        &mut self,
        old_2d_face_entity: Entity,
        new_2d_face_entity: Entity,
    ) {
        match self {
            Action::SelectShape(entity_opt) => match entity_opt {
                Some((face_2d_entity, CanvasShape::Face)) => {
                    if *face_2d_entity == old_2d_face_entity {
                        *face_2d_entity = new_2d_face_entity;
                    }
                }
                _ => {}
            },
            Action::CreateVertex(vertex_type_data, _, _) => {
                vertex_type_data.migrate_face_entities(old_2d_face_entity, new_2d_face_entity);
            }
            Action::CreateEdge(_, _, _, faces_to_create_opt, _) => {
                if let Some(entities) = faces_to_create_opt {
                    for (_, face_2d_entity, _) in entities {
                        if *face_2d_entity == old_2d_face_entity {
                            *face_2d_entity = new_2d_face_entity;
                        }
                    }
                }
            }
            Action::DeleteFace(face_2d_entity) => {
                if *face_2d_entity == old_2d_face_entity {
                    *face_2d_entity = new_2d_face_entity;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn execute(self, world: &mut World, action_stack: &mut ActionStack) -> Vec<Action> {
        match self {
            Action::SelectEntries(file_entities) => select_entries::execute(world, file_entities),
            Action::NewEntry(
                parent_entity_opt,
                new_file_name,
                entry_kind,
                old_entity_opt,
                entry_contents_opt,
            ) => new_entry::execute(
                world,
                action_stack,
                parent_entity_opt,
                new_file_name,
                entry_kind,
                old_entity_opt,
                entry_contents_opt,
            ),
            Action::DeleteEntry(file_entity, files_to_select_opt) => {
                delete_entry::execute(world, file_entity, files_to_select_opt)
            }
            Action::RenameEntry(file_entity, new_name) => {
                rename_entry::execute(world, file_entity, new_name)
            }
            Action::SelectShape(shape_2d_entity_opt) => {
                select_shape::execute(world, shape_2d_entity_opt)
            }
            Action::CreateVertex(vertex_type_data, position, old_vertex_entities_opt) => {
                create_vertex::execute(
                    world,
                    action_stack,
                    vertex_type_data,
                    position,
                    old_vertex_entities_opt,
                )
            }
            Action::DeleteVertex(vertex_2d_entity, vertex_2d_to_select_opt) => {
                delete_vertex::execute(world, vertex_2d_entity, vertex_2d_to_select_opt)
            }
            Action::MoveVertex(vertex_2d_entity, old_position, new_position) => {
                move_vertex::execute(world, vertex_2d_entity, old_position, new_position)
            }
            Action::CreateEdge(
                vertex_2d_entity_a,
                vertex_2d_entity_b,
                (shape_2d_entity_to_select, shape_2d_type_to_select),
                face_to_create_opt,
                old_edge_entities_opt,
            ) => create_edge::execute(
                world,
                action_stack,
                vertex_2d_entity_a,
                vertex_2d_entity_b,
                (shape_2d_entity_to_select, shape_2d_type_to_select),
                face_to_create_opt,
                old_edge_entities_opt,
            ),
            Action::DeleteEdge(edge_2d_entity, shape_2d_to_select_opt) => {
                delete_edge::execute(world, edge_2d_entity, shape_2d_to_select_opt)
            }
            Action::DeleteFace(face_2d_entity) => delete_face::execute(world, face_2d_entity),
        }
    }
}
