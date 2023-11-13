use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::World,
    system::{Commands, SystemState, Query, ResMut},
};
use bevy_log::info;

use naia_bevy_client::{Client, CommandsExt};

use math::Vec2;
use render_api::{Assets, base::{CpuMaterial, CpuMesh}, components::Transform};

use crate::app::resources::{icon_manager::IconShapeData, icon_data::IconFaceKey, action::icon::IconAction, icon_manager::IconManager};

pub fn execute(world: &mut World, icon_manager: &mut IconManager, current_file_entity: Entity, action: IconAction) -> Vec<IconAction> {
    let IconAction::InsertFrame(file_entity, frame_index, content_opt) = action else {
        panic!("Expected InsertFrame");
    };

    info!(
        "InsertFrame({:?}, {:?}, {:?})",
        file_entity, frame_index, content_opt
    );

    let last_frame_index: usize;
    let new_frame_entity: Entity;
    let mut entities_to_release = Vec::new();

    {
        let mut system_state: SystemState<(
            Commands,
            Client,
            ResMut<Assets<CpuMesh>>,
            ResMut<Assets<CpuMaterial>>,
            Query<&Transform>
        )> = SystemState::new(world);
        let (
            mut commands,
            mut client,
            mut meshes,
            mut materials,
            transform_q
        ) = system_state.get_mut(world);

        last_frame_index = icon_manager.current_frame_index();
        info!("current frame index: {}", last_frame_index);

        let last_frame_entity = icon_manager
            .get_frame_entity(&file_entity, last_frame_index)
            .unwrap();
        commands
            .entity(last_frame_entity)
            .release_authority(&mut client);

        new_frame_entity = icon_manager.framing_insert_frame(
            &mut commands,
            &mut client,
            file_entity,
            frame_index,
        );

        if let Some(content) = content_opt {

            let mut vertex_map = HashMap::new();
            let mut vertex_count: usize = 0;

            let mut edge_map = HashMap::new();
            let mut edge_count: usize = 0;

            for shape_data in content {
                match shape_data {
                    IconShapeData::Vertex(x, y) => {
                        let new_vertex_entity = icon_manager.create_networked_vertex(
                            &mut commands,
                            &mut client,
                            &mut meshes,
                            &mut materials,
                            &current_file_entity,
                            &new_frame_entity,
                            Vec2::new(x as f32, y as f32),
                            &mut entities_to_release,
                        );
                        vertex_map.insert(vertex_count, new_vertex_entity);
                        vertex_count += 1;
                    }
                    IconShapeData::Edge(vertex_a_index, vertex_b_index) => {
                        let vertex_a_entity = *vertex_map.get(&vertex_a_index).unwrap();
                        let vertex_b_entity = *vertex_map.get(&vertex_b_index).unwrap();

                        let new_edge_entity = icon_manager.create_networked_edge(
                            &mut commands,
                            &mut client,
                            &mut meshes,
                            &mut materials,
                            &vertex_a_entity,
                            &vertex_b_entity,
                            &current_file_entity,
                            &new_frame_entity,
                            &mut entities_to_release,
                        );
                        edge_map.insert(edge_count, new_edge_entity);
                        edge_count += 1;
                    }
                    IconShapeData::Face(palette_color_entity, vertex_a_index, vertex_b_index, vertex_c_index, edge_a_index, edge_b_index, edge_c_index) => {

                        let vertex_a_entity = *vertex_map.get(&vertex_a_index).unwrap();
                        let vertex_b_entity = *vertex_map.get(&vertex_b_index).unwrap();
                        let vertex_c_entity = *vertex_map.get(&vertex_c_index).unwrap();
                        let edge_a_entity = *edge_map.get(&edge_a_index).unwrap();
                        let edge_b_entity = *edge_map.get(&edge_b_index).unwrap();
                        let edge_c_entity = *edge_map.get(&edge_c_index).unwrap();

                        let face_key = IconFaceKey::new(vertex_a_entity, vertex_b_entity, vertex_c_entity);

                        let new_face_entity = icon_manager.create_networked_face(
                            &mut commands,
                            &mut client,
                            &mut meshes,
                            &mut materials,
                            &transform_q,
                            &face_key,
                            [edge_a_entity, edge_b_entity, edge_c_entity],
                            &current_file_entity,
                            &new_frame_entity,
                            &palette_color_entity,
                        );
                        entities_to_release.push(new_face_entity);
                    }
                }
            }
        }

        icon_manager.set_current_frame_index(frame_index);

        // TODO: migrate undo/redo entities

        system_state.apply(world);
    }

    {
        let mut system_state: SystemState<(Commands, Client)> = SystemState::new(world);
        let (mut commands, mut client) = system_state.get_mut(world);

        for entity in entities_to_release {
            commands.entity(entity).release_authority(&mut client);
        }

        system_state.apply(world);
    }

    return vec![IconAction::DeleteFrame(file_entity, frame_index)];
}
