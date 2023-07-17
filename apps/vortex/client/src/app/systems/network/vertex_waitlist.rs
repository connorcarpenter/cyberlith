use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    system::{Commands, Query},
};
use bevy_log::info;

use render_api::{Assets, base::{Color, CpuMaterial, CpuMesh}, components::RenderObjectBundle};
use vortex_proto::components::{Vertex3d, VertexRootChild};

use crate::app::{components::Vertex2d, resources::canvas_manager::CanvasManager};

pub enum VertexWaitlistInsert {
    Position,
    Parent(bool),
}

pub struct VertexWaitlistEntry {
    has_pos: bool,
    parent: Option<bool>,
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

    fn set_parent(&mut self, is_root: bool) {
        self.parent = Some(is_root);
    }

    fn set_pos(&mut self) {
        self.has_pos = true;
    }

    pub(crate) fn decompose(self) -> bool {
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
    vertex_query: &Query<&Vertex3d>,
) {
    if !vertex_waiting_entities.contains_key(&entity) {
        vertex_waiting_entities.insert(*entity, VertexWaitlistEntry::new());
    }
    let waitlist = vertex_waiting_entities.get_mut(&entity).unwrap();

    match insert {
        VertexWaitlistInsert::Position => {
            waitlist.set_pos();
        }
        VertexWaitlistInsert::Parent(is_root) => {
            waitlist.set_parent(is_root);
        }
    }

    if waitlist.is_ready() {
        let entry = vertex_waiting_entities.remove(entity).unwrap();
        vertex_process_insert_complete(
            commands,
            entry,
            *entity,
            canvas_manager,
            meshes,
            materials,
            vertex_query,
        );
    }
}

fn vertex_process_insert_complete(
    commands: &mut Commands,
    entry: VertexWaitlistEntry,
    vertex_3d_entity: Entity,
    canvas_manager: &mut CanvasManager,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    vertex_query: &Query<&Vertex3d>,
) {
    let is_root = entry.decompose();

    let vertex_3d = vertex_query.get(vertex_3d_entity).unwrap();

    commands.entity(vertex_3d_entity)
        .insert(RenderObjectBundle::sphere(
            meshes,
            materials,
            vertex_3d.x() as f32,
            vertex_3d.y() as f32,
            vertex_3d.z() as f32,
            Vertex2d::RADIUS,
            Vertex2d::SUBDIVISIONS,
            Color::GREEN,
        ))
        .insert(canvas_manager.layer_3d);

    let vertex_2d_entity = commands
        .spawn(RenderObjectBundle::circle(
            meshes,
            materials,
            vertex_3d.x() as f32,
            vertex_3d.y() as f32,
            Vertex2d::RADIUS,
            Vertex2d::SUBDIVISIONS,
            Color::GREEN,
            if is_root { Some(1) } else { None },
        ))
        .insert(canvas_manager.layer_2d)
        .insert(Vertex2d)
        .id();

    if is_root {
        commands.entity(vertex_2d_entity).insert(VertexRootChild);
    }

    info!("created Vertex3d: `{:?}`, created 2d entity: {:?}, is_root: {:?}", vertex_3d_entity, vertex_2d_entity, is_root);

    canvas_manager.register_3d_vertex(vertex_3d_entity, vertex_2d_entity);
}
