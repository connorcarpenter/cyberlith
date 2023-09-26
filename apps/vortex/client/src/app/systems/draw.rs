
use bevy_ecs::{system::{Query, ResMut}, query::With, entity::Entity,
               query::Without,
               system::Res};
use bevy_log::warn;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{
        AmbientLight, Camera, DirectionalLight, PointLight, Projection, RenderLayer, Transform,
        Visibility,
    },
    resources::RenderFrame,
    Handle,
};
use vortex_proto::components::{Edge3d, FileExtension, ShapeName, Vertex3d, VertexRoot};

use crate::app::{components::DefaultDraw, resources::{edge_manager::EdgeManager, file_manager::FileManager, tab_manager::TabManager,
                             vertex_manager::VertexManager}};

pub fn draw(
    mut render_frame: ResMut<RenderFrame>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
    // Objects
    objects_q: Query<(
        &Handle<CpuMesh>,
        &Handle<CpuMaterial>,
        &Transform,
        &Visibility,
        Option<&RenderLayer>,
    ), With<DefaultDraw>>,
    // Lights
    ambient_lights_q: Query<(&Handle<AmbientLight>, Option<&RenderLayer>)>,
    point_lights_q: Query<(&PointLight, Option<&RenderLayer>)>,
    directional_lights_q: Query<(&Handle<DirectionalLight>, Option<&RenderLayer>)>,
) {
    // Aggregate Cameras
    for (camera, transform, projection, render_layer_opt) in cameras_q.iter() {
        if !camera.is_active {
            continue;
        }
        render_frame.draw_camera(render_layer_opt, camera, transform, projection);
    }

    // Aggregate Point Lights
    for (point_light, render_layer_opt) in point_lights_q.iter() {
        render_frame.draw_point_light(render_layer_opt, point_light);
    }

    // Aggregate Directional Lights
    for (handle, render_layer_opt) in directional_lights_q.iter() {
        render_frame.draw_directional_light(render_layer_opt, handle);
    }

    // Aggregate Ambient Lights
    for (handle, render_layer_opt) in ambient_lights_q.iter() {
        render_frame.draw_ambient_light(render_layer_opt, handle);
    }

    // Aggregate RenderObjects
    for (mesh_handle, mat_handle, transform, visibility, render_layer_opt) in objects_q.iter() {
        if !visibility.visible {
            continue;
        }
        render_frame.draw_object(render_layer_opt, mesh_handle, mat_handle, transform);
    }
}

pub fn draw_vertices_and_edges(
    mut render_frame: ResMut<RenderFrame>,
    file_manager: Res<FileManager>,
    tab_manager: Res<TabManager>,
    vertex_manager: Res<VertexManager>,
    edge_manager: Res<EdgeManager>,

    // Objects
    objects_q: Query<(
        &Handle<CpuMesh>,
        &Transform,
        Option<&RenderLayer>,
    )>,
    vertices_q: Query<(Entity, &Visibility, Option<&ShapeName>, Option<&VertexRoot>), (With<Vertex3d>, Without<DefaultDraw>)>,
    edges_q: Query<(Entity, &Visibility), (With<Edge3d>, Without<DefaultDraw>)>,
) {
    let Some(current_tab) = tab_manager.current_tab_entity() else {
        return;
    };
    let current_file = file_manager.get_file_type(current_tab);

    // draw vertices
    for (vertex_3d_entity, visibility, shape_name_opt, vertex_root_opt) in vertices_q.iter() {
        if !visibility.visible {
            continue;
        }

        // draw 3d vertex
        let Ok((mesh_handle, transform, render_layer_opt)) = objects_q.get(vertex_3d_entity) else {
            warn!("vertex 3d entity query {:?} not found", vertex_3d_entity);
            continue;
        };

        let mat_handle = get_shape_color(&vertex_manager, current_file, shape_name_opt, vertex_root_opt.is_some());

        render_frame.draw_object(render_layer_opt, mesh_handle, &mat_handle, transform);

        // draw vertex 2d
        let Some(vertex_2d_entity) = vertex_manager.vertex_entity_3d_to_2d(&vertex_3d_entity) else {continue};

        let (mesh_handle, transform, render_layer_opt) = objects_q.get(vertex_2d_entity).unwrap();
        render_frame.draw_object(render_layer_opt, mesh_handle, &mat_handle, transform);
    }

    // draw edges
    for (edge_3d_entity, visibility) in edges_q.iter() {
        if !visibility.visible {
            continue;
        }

        // draw 3d edge
        let (mesh_handle, transform, render_layer_opt) = objects_q.get(edge_3d_entity).unwrap();

        let (_, end_vertex_3d_entity) = edge_manager.edge_get_endpoints(&edge_3d_entity);
        let (_, _, shape_name_opt, vertex_root_opt) = vertices_q.get(end_vertex_3d_entity).unwrap();
        let mat_handle = get_shape_color(&vertex_manager, current_file, shape_name_opt, vertex_root_opt.is_some());

        render_frame.draw_object(render_layer_opt, mesh_handle, &mat_handle, transform);

        // draw edge 2d
        let Some(edge_2d_entity) = edge_manager.edge_entity_3d_to_2d(&edge_3d_entity) else {continue};

        let (mesh_handle, transform, render_layer_opt) = objects_q.get(edge_2d_entity).unwrap();
        render_frame.draw_object(render_layer_opt, mesh_handle, &mat_handle, transform);
    }
}

fn get_shape_color(vertex_manager: &Res<VertexManager>, current_file: FileExtension, shape_name_opt: Option<&ShapeName>, vertex_is_root: bool) -> Handle<CpuMaterial> {
    match current_file {
        FileExtension::Anim => {
            if vertex_is_root {
                vertex_manager.mat_root_vertex
            } else {
                let can_rotate = if let Some(shape_name) = shape_name_opt {
                    if shape_name.value.len() > 0 {
                        true
                    } else {
                        false
                    }
                } else {
                    false
                };
                if can_rotate {
                    vertex_manager.mat_enabled_vertex
                } else {
                    vertex_manager.mat_disabled_vertex
                }
            }
        }
        _ => {
            if vertex_is_root {
                vertex_manager.mat_root_vertex
            } else {
                vertex_manager.mat_enabled_vertex
            }
        }
    }
}
