use bevy_ecs::{
    entity::Entity,
    query::{With, Without},
    system::{Query, Res, ResMut, SystemState},
    world::{Mut, World},
};
use logging::warn;

use render_api::{
    base::{CpuMaterial, CpuMesh},
    components::{
        AmbientLight, Camera, DirectionalLight, PointLight, Projection, RenderLayer, Transform,
        Visibility,
    },
    resources::RenderFrame,
};
use storage::Handle;

use editor_proto::components::{Edge3d, FileExtension, ShapeName, Vertex3d, VertexRoot};

use crate::app::{
    components::DefaultDraw,
    resources::{
        animation_manager::AnimationManager, edge_manager::edge_is_enabled,
        edge_manager::EdgeManager, file_manager::FileManager, icon_manager::IconManager,
        model_manager::ModelManager, tab_manager::TabManager, vertex_manager::VertexManager,
    },
};

pub fn draw(
    file_manager: Res<FileManager>,
    tab_manager: Res<TabManager>,
    animation_manager: Res<AnimationManager>,
    mut render_frame: ResMut<RenderFrame>,
    // Cameras
    cameras_q: Query<(&Camera, &Transform, &Projection, Option<&RenderLayer>)>,
    // Objects
    objects_q: Query<
        (
            &Handle<CpuMesh>,
            &Handle<CpuMaterial>,
            &Transform,
            &Visibility,
            Option<&RenderLayer>,
        ),
        With<DefaultDraw>,
    >,
    // Lights
    ambient_lights_q: Query<(&AmbientLight, Option<&RenderLayer>)>,
    point_lights_q: Query<(&PointLight, Option<&RenderLayer>)>,
    directional_lights_q: Query<(&DirectionalLight, Option<&RenderLayer>)>,
) {
    // Aggregate Cameras
    for (camera, transform, projection, render_layer_opt) in cameras_q.iter() {
        if !camera.is_active {
            continue;
        }
        render_frame.draw_camera(render_layer_opt, camera, transform, projection);
    }

    // Aggregate Point Lights
    for (light, render_layer_opt) in point_lights_q.iter() {
        render_frame.draw_point_light(render_layer_opt, light);
    }

    // Aggregate Directional Lights
    for (light, render_layer_opt) in directional_lights_q.iter() {
        render_frame.draw_directional_light(render_layer_opt, light);
    }

    // Aggregate Ambient Lights
    for (light, render_layer_opt) in ambient_lights_q.iter() {
        render_frame.draw_ambient_light(render_layer_opt, light);
    }

    let Some(current_file_entity) = tab_manager.current_tab_entity() else {
        return;
    };
    let current_file_type = file_manager.get_file_type(&current_file_entity);
    match current_file_type {
        FileExtension::Anim => {
            if animation_manager.is_framing() {
                return;
            }
        }
        FileExtension::Model | FileExtension::Scene | FileExtension::Icon => {
            return;
        }
        _ => {}
    }

    // Aggregate RenderObjects
    for (mesh_handle, mat_handle, transform, visibility, render_layer_opt) in objects_q.iter() {
        if !visibility.visible {
            continue;
        }
        render_frame.draw_mesh(render_layer_opt, mesh_handle, mat_handle, transform);
    }
}

pub fn draw_vertices_and_edges(world: &mut World) {
    let Some(current_tab) = world
        .get_resource::<TabManager>()
        .unwrap()
        .current_tab_entity()
    else {
        return;
    };
    let file_ext = world
        .get_resource::<FileManager>()
        .unwrap()
        .get_file_type(current_tab);
    let current_file_entity = *current_tab;

    match file_ext {
        FileExtension::Anim => {
            if world
                .get_resource::<AnimationManager>()
                .unwrap()
                .is_framing()
            {
                world.resource_scope(|world, mut animation_manager: Mut<AnimationManager>| {
                    animation_manager.draw_framing(world);
                });

                return;
            }
        }
        FileExtension::Model | FileExtension::Scene => {
            world.resource_scope(|world, model_manager: Mut<ModelManager>| {
                model_manager.draw(world, &file_ext, &current_file_entity);
            });

            return;
        }
        FileExtension::Icon => {
            world.resource_scope(|world, mut icon_manager: Mut<IconManager>| {
                icon_manager.draw(world, &current_file_entity);
            });

            return;
        }
        _ => {}
    }

    draw_vertices_and_edges_inner(world, file_ext);
}

fn draw_vertices_and_edges_inner(world: &mut World, current_file: FileExtension) {
    let mut system_state: SystemState<(
        ResMut<RenderFrame>,
        Res<VertexManager>,
        Res<EdgeManager>,
        Res<AnimationManager>,
        Query<(&Handle<CpuMesh>, &Transform, Option<&RenderLayer>)>,
        Query<
            (Entity, &Visibility, Option<&ShapeName>, Option<&VertexRoot>),
            (With<Vertex3d>, Without<DefaultDraw>),
        >,
        Query<(Entity, &Visibility), (With<Edge3d>, Without<DefaultDraw>)>,
        Query<&Handle<CpuMaterial>>,
    )> = SystemState::new(world);
    let (
        mut render_frame,
        vertex_manager,
        edge_manager,
        animation_manager,
        objects_q,
        vertices_q,
        edges_q,
        materials_q,
    ) = system_state.get_mut(world);

    let mut edge_angles_are_visible = edge_manager.edge_angles_are_visible(current_file);
    if current_file == FileExtension::Anim {
        if animation_manager.preview_frame_selected() {
            edge_angles_are_visible = false;
        }
    }
    let must_check_edge_enabled =
        current_file == FileExtension::Anim && !animation_manager.preview_frame_selected();

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

        let edge_is_enabled = if must_check_edge_enabled {
            edge_is_enabled(shape_name_opt)
        } else {
            true
        };
        let mat_handle = get_shape_color(
            &vertex_manager,
            current_file,
            vertex_root_opt.is_some(),
            edge_is_enabled,
        );

        // can't we ONLY draw this when 3d mode is enabled?
        render_frame.draw_mesh(render_layer_opt, mesh_handle, &mat_handle, transform);

        // draw vertex 2d
        let Some(vertex_2d_entity) = vertex_manager.vertex_entity_3d_to_2d(&vertex_3d_entity)
        else {
            continue;
        };

        let (mesh_handle, transform, render_layer_opt) = objects_q.get(vertex_2d_entity).unwrap();
        // can't we ONLY draw this when 2d mode is enabled?
        render_frame.draw_mesh(render_layer_opt, mesh_handle, &mat_handle, transform);
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
        let edge_is_enabled = if must_check_edge_enabled {
            edge_is_enabled(shape_name_opt)
        } else {
            true
        };
        let mat_handle = get_shape_color(
            &vertex_manager,
            current_file,
            vertex_root_opt.is_some(),
            edge_is_enabled,
        );

        render_frame.draw_mesh(render_layer_opt, mesh_handle, &mat_handle, transform);

        // draw edge 2d
        let Some(edge_2d_entity) = edge_manager.edge_entity_3d_to_2d(&edge_3d_entity) else {
            continue;
        };

        let (mesh_handle, transform, render_layer_opt) = objects_q.get(edge_2d_entity).unwrap();
        render_frame.draw_mesh(render_layer_opt, mesh_handle, &mat_handle, transform);

        if edge_angles_are_visible && edge_is_enabled {
            // draw edge angles
            edge_manager.draw_edge_angles(
                &edge_3d_entity,
                &mut render_frame,
                &objects_q,
                &materials_q,
            );
        }
    }
}

fn get_shape_color(
    vertex_manager: &Res<VertexManager>,
    current_file: FileExtension,
    vertex_is_root: bool,
    edge_is_enabled: bool,
) -> Handle<CpuMaterial> {
    if vertex_is_root {
        vertex_manager.mat_root_vertex
    } else {
        match current_file {
            FileExtension::Anim => {
                if edge_is_enabled {
                    vertex_manager.mat_enabled_vertex
                } else {
                    vertex_manager.mat_disabled_vertex
                }
            }
            _ => vertex_manager.mat_enabled_vertex,
        }
    }
}
