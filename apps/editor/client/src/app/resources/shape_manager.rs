use bevy_ecs::entity::Entity;
use logging::info;

use crate::app::resources::{
    edge_manager::EdgeManager, face_manager::FaceManager, file_manager::FileManager,
    shape_data::CanvasShape, vertex_manager::VertexManager,
};

pub struct ShapeManager;

impl ShapeManager {
    pub(crate) fn shape_entity_2d_to_3d(
        vertex_manager: &VertexManager,
        edge_manager: &EdgeManager,
        face_manager: &FaceManager,
        entity_2d: &Entity,
        shape_type: CanvasShape,
    ) -> Option<Entity> {
        match shape_type {
            CanvasShape::RootVertex | CanvasShape::Vertex => {
                vertex_manager.vertex_entity_2d_to_3d(entity_2d)
            }
            CanvasShape::Edge => {
                let output = edge_manager.edge_entity_2d_to_3d(entity_2d);
                info!("edge entity 2d `{:?}` to 3d: `{:?}`", entity_2d, output);
                output
            }
            CanvasShape::Face => face_manager.face_entity_2d_to_3d(entity_2d),
        }
    }

    pub(crate) fn shape_entity_3d_to_2d(
        vertex_manager: &VertexManager,
        edge_manager: &EdgeManager,
        face_manager: &FaceManager,
        entity_3d: &Entity,
    ) -> Option<Entity> {
        let shape_type =
            Self::shape_type_from_3d_entity(vertex_manager, edge_manager, face_manager, entity_3d)
                .unwrap();

        match shape_type {
            CanvasShape::RootVertex | CanvasShape::Vertex => {
                vertex_manager.vertex_entity_3d_to_2d(entity_3d)
            }
            CanvasShape::Edge => edge_manager.edge_entity_3d_to_2d(entity_3d),
            CanvasShape::Face => face_manager.face_entity_3d_to_2d(entity_3d),
        }
    }

    fn shape_type_from_3d_entity(
        vertex_manager: &VertexManager,
        edge_manager: &EdgeManager,
        face_manager: &FaceManager,
        entity_3d: &Entity,
    ) -> Option<CanvasShape> {
        if vertex_manager.has_vertex_entity_3d(entity_3d) {
            Some(CanvasShape::Vertex)
        } else if edge_manager.has_edge_entity_3d(entity_3d) {
            Some(CanvasShape::Edge)
        } else if face_manager.has_face_entity_3d(entity_3d) {
            Some(CanvasShape::Face)
        } else {
            None
        }
    }

    // returns true if shape is owned by tab
    pub(crate) fn is_owned_by_file(
        file_manager: &FileManager,
        file_entity: &Entity,
        content_file_entity_opt: Option<&Entity>,
    ) -> bool {
        match Self::is_owned_by_file_internal(file_manager, file_entity, content_file_entity_opt) {
            ContentOwnership::OwnedByTab => true,
            ContentOwnership::UnownedByTab => false,
            ContentOwnership::Unowned => false,
            ContentOwnership::DependencyOwnedByTab => true,
        }
    }

    // returns true if shape is owned by tab
    fn is_owned_by_file_internal(
        file_manager: &FileManager,
        file_entity: &Entity,
        content_file_entity_opt: Option<&Entity>,
    ) -> ContentOwnership {
        if let Some(content_file_entity) = content_file_entity_opt {
            if content_file_entity == file_entity {
                return ContentOwnership::OwnedByTab;
            } else {
                // check if file is a dependency of owning file
                return match file_manager.file_has_dependency(file_entity, content_file_entity) {
                    true => ContentOwnership::DependencyOwnedByTab,
                    false => ContentOwnership::UnownedByTab,
                };
            }
        }
        return ContentOwnership::Unowned;
    }
}

enum ContentOwnership {
    DependencyOwnedByTab,
    OwnedByTab,
    UnownedByTab,
    Unowned,
}
