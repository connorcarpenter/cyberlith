use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Commands};

use math::{Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::RenderObjectBundle,
    Assets,
};
use vortex_proto::components::VertexRootChild;

use crate::app::{
    components::{Edge2d, Edge3d, Vertex2d},
    resources::canvas_manager::CanvasManager,
    shapes::{create_2d_edge_arrow, create_3d_edge_diamond},
};

pub enum VertexWaitlistInsert {
    Position,
    Parent(Option<Entity>),
}

pub struct VertexWaitlistEntry {
    has_pos: bool,
    parent: Option<Option<Entity>>,
}

impl VertexWaitlistEntry {
    fn new() -> Self {
        Self {
            has_pos: false,
            parent: None,
        }
    }

    fn is_ready(&self) -> bool {
        self.has_pos && self.parent.is_some()
    }

    fn set_parent(&mut self, parent: Option<Entity>) {
        self.parent = Some(parent);
    }

    fn set_position(&mut self) {
        self.has_pos = true;
    }

    pub(crate) fn decompose(self) -> Option<Entity> {
        return self.parent.unwrap();
    }
}

pub fn vertex_process_insert(
    vertex_waiting_entities: &mut HashMap<Entity, VertexWaitlistEntry>,
    commands: &mut Commands,
    insert: VertexWaitlistInsert,
    entity: &Entity,
    canvas_manager: &mut CanvasManager,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
) {
    if !vertex_waiting_entities.contains_key(&entity) {
        vertex_waiting_entities.insert(*entity, VertexWaitlistEntry::new());
    }
    let waitlist = vertex_waiting_entities.get_mut(&entity).unwrap();

    match insert {
        VertexWaitlistInsert::Position => {
            waitlist.set_position();
        }
        VertexWaitlistInsert::Parent(parent) => {
            waitlist.set_parent(parent);
        }
    }

    if waitlist.is_ready() {
        let entry = vertex_waiting_entities.remove(entity).unwrap();
        vertex_process_insert_complete(commands, entry, *entity, canvas_manager, meshes, materials);
    }
}

fn vertex_process_insert_complete(
    commands: &mut Commands,
    entry: VertexWaitlistEntry,
    vertex_3d_entity: Entity,
    canvas_manager: &mut CanvasManager,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
) {
    let parent_3d_entity_opt = entry.decompose();

    let color = match parent_3d_entity_opt {
        Some(_) => Vertex2d::CHILD_COLOR,
        None => Vertex2d::ROOT_COLOR,
    };

    vertex_3d_postprocess(
        commands,
        canvas_manager,
        meshes,
        materials,
        parent_3d_entity_opt,
        vertex_3d_entity,
        color,
    );
}

pub fn vertex_3d_postprocess(
    commands: &mut Commands,
    canvas_manager: &mut CanvasManager,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    parent_3d_entity_opt: Option<Entity>,
    vertex_3d_entity: Entity,
    color: Color,
) -> Entity {

    commands
        .entity(vertex_3d_entity)
        .insert(RenderObjectBundle::sphere(
            meshes,
            materials,
            Vec3::ZERO,
            Vertex2d::RADIUS,
            Vertex2d::SUBDIVISIONS,
            color,
        ))
        .insert(canvas_manager.layer_3d);

    let vertex_2d_entity = commands
        .spawn(RenderObjectBundle::circle(
            meshes,
            materials,
            Vec2::ZERO,
            Vertex2d::RADIUS,
            Vertex2d::SUBDIVISIONS,
            color,
            None,
        ))
        .insert(canvas_manager.layer_2d)
        .insert(Vertex2d)
        .id();

    if let Some(parent_3d_entity) = parent_3d_entity_opt {
        // create 2d edge entity
        commands
            .spawn(create_2d_edge_arrow(
                meshes,
                materials,
                Vec2::ZERO,
                Vec2::X,
                color,
            ))
            .insert(canvas_manager.layer_2d)
            .insert(Edge2d::new(vertex_2d_entity, parent_3d_entity));

        // create 3d edge entity
        commands
            .spawn(create_3d_edge_diamond(
                meshes,
                materials,
                Vec3::ZERO,
                Vec3::X,
                color,
            ))
            .insert(canvas_manager.layer_3d)
            .insert(Edge3d::new(vertex_3d_entity, parent_3d_entity));
    } else {
        commands.entity(vertex_2d_entity).insert(VertexRootChild);
    }

    // info!(
    //     "created Vertex3d: `{:?}`, created 2d entity: {:?}, parent: {:?}",
    //     vertex_3d_entity, vertex_2d_entity, parent_entity_opt
    // );

    canvas_manager.register_3d_vertex(vertex_3d_entity, vertex_2d_entity);
    canvas_manager.recalculate_3d_view();

    vertex_2d_entity
}
