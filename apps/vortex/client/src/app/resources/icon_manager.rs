use std::collections::HashSet;

use bevy_ecs::{
    entity::Entity,
    query::With,
    system::Res,
    system::{Query, ResMut, Resource, SystemState},
    world::World,
};
use bevy_log::warn;

use math::{convert_3d_to_2d, Vec3};

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{Camera, CameraProjection, Projection, RenderLayer, Transform},
    resources::RenderFrame,
    Handle,
};

use vortex_proto::components::{Edge3d, EdgeAngle, ShapeName, Vertex3d};

use crate::app::{
    components::{
        Edge2dLocal, Edge3dLocal, EdgeAngleLocal, LocalShape,
        OwnedByFileLocal, Vertex2d,
    },
    resources::{
        compass::Compass, edge_manager::EdgeManager,
        file_manager::FileManager, grid::Grid, input::InputManager,
        shape_data::CanvasShape, tab_manager::TabManager, vertex_manager::VertexManager,
    },
};

#[derive(Resource)]
pub struct IconManager {

}

impl Default for IconManager {
    fn default() -> Self {
        Self {

        }
    }
}

impl IconManager {

    pub fn sync_shapes(
        &mut self,
        world: &mut World,
        vertex_manager: &VertexManager,
        file_entity: &Entity,
        camera_3d_entity: &Entity,
        camera_is_2d: bool,
        camera_3d_scale: f32,
    ) {
        let (
            vertex_3d_entities,
            local_vertex_3d_entities,
            edge_3d_entities,
            local_edge_3d_entities,
        ) = match self.sync_3d_shapes(world, vertex_manager, &file_entity, camera_3d_scale) {
            Some(value) => value,
            None => return,
        };

        if !camera_is_2d {
            return;
        }

        Self::sync_2d_shapes(
            world,
            vertex_manager,
            camera_3d_entity,
            camera_3d_scale,
            vertex_3d_entities,
            local_vertex_3d_entities,
            edge_3d_entities,
            local_edge_3d_entities,
        );
    }

    fn sync_3d_shapes(
        &mut self,
        world: &mut World,
        vertex_manager: &VertexManager,
        file_entity: &Entity,
        camera_3d_scale: f32,
    ) -> Option<(
        HashSet<Entity>,
        HashSet<Entity>,
        HashSet<Entity>,
        HashSet<Entity>,
    )> {
        // only triggers when canvas is redrawn
        let local_vertex_3d_scale = LocalShape::VERTEX_RADIUS / camera_3d_scale;
        let local_vertex_3d_scale = Vec3::splat(local_vertex_3d_scale);
        let local_edge_3d_scale = LocalShape::EDGE_THICKNESS / camera_3d_scale;

        // gather 3D entities for Compass/Grid Vertices
        let mut vertex_3d_entities: HashSet<Entity> = HashSet::new();
        let mut local_vertex_3d_entities: HashSet<Entity> = HashSet::new();

        let compass_3d_entities = world.get_resource::<Compass>().unwrap().vertices();
        let grid_3d_entities = world.get_resource::<Grid>().unwrap().vertices();
        vertex_3d_entities.extend(compass_3d_entities);
        vertex_3d_entities.extend(grid_3d_entities);
        local_vertex_3d_entities.extend(compass_3d_entities);
        local_vertex_3d_entities.extend(grid_3d_entities);

        // from 3D vertex entities, get list of 3D edge entities
        let mut edge_3d_entities: HashSet<Entity> = HashSet::new();
        let mut local_edge_3d_entities = HashSet::new();

        for vertex_3d_entity in vertex_3d_entities.iter() {
            let Some(vertex_data) = vertex_manager.get_vertex_3d_data(vertex_3d_entity) else {
                warn!("vertex_3d_entity {:?} has no vertex_data", vertex_3d_entity);
                continue;
            };

            for edge_3d_entity in vertex_data.edges_3d.iter() {
                edge_3d_entities.insert(*edge_3d_entity);

                if local_vertex_3d_entities.contains(vertex_3d_entity) {
                    local_edge_3d_entities.insert(*edge_3d_entity);
                }
            }
        }

        // for ALL gathered 3D vertex entities, convert Vertex3D -> 3d Transform
        let mut system_state: SystemState<Query<(&Vertex3d, &mut Transform)>> =
            SystemState::new(world);
        let mut vertex_3d_q = system_state.get_mut(world);

        for vertex_3d_entity in vertex_3d_entities.iter() {
            let Ok((vertex_3d, mut transform)) = vertex_3d_q.get_mut(*vertex_3d_entity) else {
                continue;
            };
            transform.translation = vertex_3d.as_vec3();

            if local_vertex_3d_entities.contains(vertex_3d_entity) {
                transform.scale = local_vertex_3d_scale;
            }
        }

        // for ALL gathered 3D edge entities, sync with 3d vertex transforms
        let mut system_state: SystemState<(
            Query<(&Edge3dLocal, Option<&EdgeAngle>)>,
            Query<&mut Transform>,
        )> = SystemState::new(world);
        let (edge_3d_q, mut transform_q) = system_state.get_mut(world);

        for edge_3d_entity in edge_3d_entities.iter() {
            let Ok((edge_3d_local, edge_angle_opt)) = edge_3d_q.get(*edge_3d_entity) else {
                continue;
            };
            EdgeManager::sync_3d_edge(
                &mut transform_q,
                edge_3d_entity,
                edge_3d_local,
                edge_angle_opt,
            );
            if local_edge_3d_entities.contains(edge_3d_entity) {
                let mut transform = transform_q.get_mut(*edge_3d_entity).unwrap();
                transform.scale.x = local_edge_3d_scale;
                transform.scale.y = local_edge_3d_scale;
            }
        }
        Some((
            vertex_3d_entities,
            local_vertex_3d_entities,
            edge_3d_entities,
            local_edge_3d_entities,
        ))
    }

    fn sync_2d_shapes(
        world: &mut World,
        vertex_manager: &VertexManager,
        camera_3d_entity: &Entity,
        camera_3d_scale: f32,
        vertex_3d_entities: HashSet<Entity>,
        local_vertex_3d_entities: HashSet<Entity>,
        edge_3d_entities: HashSet<Entity>,
        local_edge_3d_entities: HashSet<Entity>,
    ) {
        // let vertex_2d_scale = Vec3::splat(LocalShape::VERTEX_RADIUS * camera_3d_scale);
        // let edge_2d_scale = LocalShape::EDGE_THICKNESS * camera_3d_scale;
        let local_vertex_2d_scale = LocalShape::VERTEX_RADIUS;
        let normal_vertex_2d_scale = Vertex2d::RADIUS * camera_3d_scale;
        let hover_vertex_2d_scale = Vertex2d::HOVER_RADIUS * camera_3d_scale;

        let local_edge_2d_scale = LocalShape::EDGE_THICKNESS;
        let normal_edge_2d_scale = Edge2dLocal::NORMAL_THICKNESS * camera_3d_scale;
        let hover_edge_2d_scale = Edge2dLocal::HOVER_THICKNESS * camera_3d_scale;

        let mut system_state: SystemState<(
            Res<InputManager>,
            Res<EdgeManager>,
            Query<(&Camera, &Projection)>,
            Query<&mut Transform>,
            Query<&Edge2dLocal>,
        )> = SystemState::new(world);
        let (
            input_manager,
            edge_manager,
            camera_q,
            mut transform_q,
            edge_2d_local_q,
        ) = system_state.get_mut(world);

        let Ok((camera, camera_projection)) = camera_q.get(*camera_3d_entity) else {
            return;
        };
        let Ok(camera_transform) = transform_q.get(*camera_3d_entity) else {
            return;
        };
        let camera_viewport = camera.viewport.unwrap();
        let view_matrix = camera_transform.view_matrix();
        let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

        // for ALL gathered 2D vertex entities, derive 2d transform from 3d transform
        for vertex_3d_entity in vertex_3d_entities.iter() {
            let Some(vertex_data) = vertex_manager.get_vertex_3d_data(vertex_3d_entity) else {
                warn!("vertex_3d_entity {:?} has no vertex_data", vertex_3d_entity);
                continue;
            };
            let vertex_2d_entity = vertex_data.entity_2d;

            // get 3d transform
            let Ok(vertex_3d_transform) = transform_q.get(*vertex_3d_entity) else {
                warn!("Vertex3d entity {:?} has no Transform", vertex_3d_entity);
                continue;
            };
            // derive 2d transform from 3d transform
            let (coords, depth) = convert_3d_to_2d(
                &view_matrix,
                &projection_matrix,
                &camera_viewport.size_vec2(),
                &vertex_3d_transform.translation,
            );

            // get 2d transform
            let Ok(mut vertex_2d_transform) = transform_q.get_mut(vertex_2d_entity) else {
                panic!("Vertex2d entity {:?} has no Transform", vertex_2d_entity);
            };
            vertex_2d_transform.translation.x = coords.x;
            vertex_2d_transform.translation.y = coords.y;
            vertex_2d_transform.translation.z = depth;

            if local_vertex_3d_entities.contains(vertex_3d_entity) {
                vertex_2d_transform.scale.x = local_vertex_2d_scale;
                vertex_2d_transform.scale.y = local_vertex_2d_scale;
            } else {
                vertex_2d_transform.scale.x = normal_vertex_2d_scale;
                vertex_2d_transform.scale.y = normal_vertex_2d_scale;
                if let Some((hover_entity, CanvasShape::Vertex)) = input_manager.hovered_entity {
                    if hover_entity == vertex_2d_entity {
                        vertex_2d_transform.scale.x = hover_vertex_2d_scale;
                        vertex_2d_transform.scale.y = hover_vertex_2d_scale;
                    }
                }
            }
        }

        // for ALL gathered 2D edge entities, derive 2d transform from 2d vertex data
        for edge_3d_entity in edge_3d_entities.iter() {
            let Some(edge_2d_entity) = edge_manager.edge_entity_3d_to_2d(edge_3d_entity) else {
                panic!("edge_3d_entity {:?} has no edge_2d_entity", edge_3d_entity);
            };

            // derive 2d transform from 2d vertex data
            let Ok(edge_endpoints) = edge_2d_local_q.get(edge_2d_entity) else {
                warn!("Edge2d entity {:?} has no Edge2dLocal", edge_2d_entity);
                continue;
            };
            EdgeManager::sync_2d_edge(&mut transform_q, &edge_2d_entity, edge_endpoints);
            let mut transform = transform_q.get_mut(edge_2d_entity).unwrap();
            if local_edge_3d_entities.contains(edge_3d_entity) {
                transform.scale.y = local_edge_2d_scale;
            } else {
                transform.scale.y = normal_edge_2d_scale;
                if let Some((hover_entity, CanvasShape::Edge)) = input_manager.hovered_entity {
                    if hover_entity == edge_2d_entity {
                        transform.scale.y = hover_edge_2d_scale;
                    }
                }
            }
        }
    }

    pub fn draw(&self, world: &mut World, current_file_entity: &Entity) {
        let Some(current_tab_state) = world.get_resource::<TabManager>().unwrap().current_tab_state() else {
            return;
        };
        let camera_state = &current_tab_state.camera_state;
        let camera_is_2d = camera_state.is_2d();
        if camera_is_2d {
            self.draw_2d(world, current_file_entity);
        } else {
            self.draw_3d(world, current_file_entity);
        }
    }

    fn draw_2d(&self, world: &mut World, current_file_entity: &Entity) {
        {
            let mut vertex_3d_entities: HashSet<Entity> = HashSet::new();
            let compass_3d_entities = world.get_resource::<Compass>().unwrap().vertices();
            let grid_3d_entities = world.get_resource::<Grid>().unwrap().vertices();
            vertex_3d_entities.extend(compass_3d_entities);
            vertex_3d_entities.extend(grid_3d_entities);

            let mut edge_2d_entities = HashSet::new();

            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                Res<FileManager>,
                Res<InputManager>,
                Res<VertexManager>,
                Res<EdgeManager>,
                Query<(&Handle<CpuMesh>, &Transform, Option<&RenderLayer>)>,
                Query<&Handle<CpuMaterial>>,
                Query<(Entity, &OwnedByFileLocal), With<Edge3d>>,
                Query<Option<&ShapeName>>,
            )> = SystemState::new(world);
            let (
                mut render_frame,
                file_manager,
                input_manager,
                vertex_manager,
                edge_manager,
                objects_q,
                materials_q,
                edge_q,
                shape_name_q,
            ) = system_state.get_mut(world);

            // draw vertices (compass, grid, net transform controls)
            for vertex_3d_entity in vertex_3d_entities.iter() {
                // draw vertex 2d
                let Some(data) = vertex_manager.get_vertex_3d_data(&vertex_3d_entity) else {
                    continue;
                };

                let (mesh_handle, transform, render_layer_opt) =
                    objects_q.get(data.entity_2d).unwrap();
                let mat_handle = materials_q.get(data.entity_2d).unwrap();
                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);

                for edge_3d_entity in data.edges_3d.iter() {
                    let edge_2d_entity = edge_manager.edge_entity_3d_to_2d(edge_3d_entity).unwrap();
                    edge_2d_entities.insert(edge_2d_entity);
                }
            }

            // draw edges (compass, grid, net transform controls)
            for edge_2d_entity in edge_2d_entities.iter() {
                let (mesh_handle, transform, render_layer_opt) =
                    objects_q.get(*edge_2d_entity).unwrap();
                let mat_handle = materials_q.get(*edge_2d_entity).unwrap();
                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);
            }

            // draw select line & circle
            match input_manager.selected_shape_2d() {
                Some((_, CanvasShape::Edge)) => {
                    // draw select line
                    if let Some(select_line_entity) = input_manager.select_line_entity {
                        let (mesh_handle, transform, render_layer_opt) =
                            objects_q.get(select_line_entity).unwrap();
                        let mat_handle = materials_q.get(select_line_entity).unwrap();
                        render_frame.draw_object(
                            render_layer_opt,
                            mesh_handle,
                            &mat_handle,
                            transform,
                        );
                    }
                }
                Some((_, CanvasShape::Vertex)) => {
                    // draw select circle
                    if let Some(select_circle_entity) = input_manager.select_circle_entity {
                        let (mesh_handle, transform, render_layer_opt) =
                            objects_q.get(select_circle_entity).unwrap();
                        let mat_handle = materials_q.get(select_circle_entity).unwrap();
                        render_frame.draw_object(
                            render_layer_opt,
                            mesh_handle,
                            &mat_handle,
                            transform,
                        );
                    }
                }
                _ => {}
            }
        }
    }

    fn draw_3d(&self, world: &mut World, current_file_entity: &Entity) {
        {
            let mut vertex_3d_entities: HashSet<Entity> = HashSet::new();
            let compass_3d_entities = world.get_resource::<Compass>().unwrap().vertices();
            let grid_3d_entities = world.get_resource::<Grid>().unwrap().vertices();
            vertex_3d_entities.extend(compass_3d_entities);
            vertex_3d_entities.extend(grid_3d_entities);

            let mut edge_3d_entities = HashSet::new();

            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                Res<FileManager>,
                Res<VertexManager>,
                Res<EdgeManager>,
                Query<(
                    &Handle<CpuMesh>,
                    &Handle<CpuMaterial>,
                    &Transform,
                    Option<&RenderLayer>,
                )>,
                Query<(Entity, &OwnedByFileLocal), With<Edge3d>>,
                Query<Option<&ShapeName>>,
            )> = SystemState::new(world);
            let (
                mut render_frame,
                file_manager,
                vertex_manager,
                edge_manager,
                objects_q,
                edge_q,
                shape_name_q,
            ) = system_state.get_mut(world);

            // draw vertices (compass, grid)
            for vertex_3d_entity in vertex_3d_entities.iter() {
                // draw vertex 2d
                let Some(data) = vertex_manager.get_vertex_3d_data(&vertex_3d_entity) else { continue };

                let (mesh_handle, mat_handle, transform, render_layer_opt) =
                    objects_q.get(*vertex_3d_entity).unwrap();
                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);

                for edge_3d_entity in data.edges_3d.iter() {
                    edge_3d_entities.insert(*edge_3d_entity);
                }
            }

            // draw edges (compass, grid)
            for edge_3d_entity in edge_3d_entities.iter() {
                let (mesh_handle, mat_handle, transform, render_layer_opt) =
                    objects_q.get(*edge_3d_entity).unwrap();
                render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);
            }
        }
    }
}