use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Commands};

use math::{Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::RenderObjectBundle,
    Assets,
};
use vortex_proto::{components::{OwnedByTab, VertexRootChild}, types::TabId};

use crate::app::{
    components::{Edge2d, Edge3d, Vertex2d},
    resources::canvas_manager::CanvasManager,
    shapes::{
        create_2d_edge_arrow, create_2d_edge_line, create_3d_edge_diamond, create_3d_edge_line,
    },
};

pub enum VertexWaitlistInsert {
    Position,
    Parent(Option<Entity>),
    OwnedByTab(TabId),
}

pub struct VertexWaitlistEntry {
    has_pos: bool,
    parent: Option<Option<Entity>>,
    tab_id: Option<TabId>,
}

impl VertexWaitlistEntry {
    fn new() -> Self {
        Self {
            has_pos: false,
            parent: None,
            tab_id: None,
        }
    }

    fn is_ready(&self) -> bool {
        self.has_pos && self.parent.is_some() && self.tab_id.is_some()
    }

    fn set_parent(&mut self, parent: Option<Entity>) {
        self.parent = Some(parent);
    }

    fn set_position(&mut self) {
        self.has_pos = true;
    }

    fn set_tab_id(&mut self, tab_id: TabId) {
        self.tab_id = Some(tab_id);
    }

    pub(crate) fn decompose(self) -> (Option<Entity>, TabId) {
        return (self.parent.unwrap(), self.tab_id.unwrap());
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
        VertexWaitlistInsert::OwnedByTab(tab_id) => {
            waitlist.set_tab_id(tab_id);
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
    let (parent_3d_entity_opt, tab_id) = entry.decompose();

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
        Some(tab_id),
        color,
        true,
    );
}

pub fn vertex_3d_postprocess(
    commands: &mut Commands,
    canvas_manager: &mut CanvasManager,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    parent_3d_entity_opt: Option<Entity>,
    vertex_3d_entity: Entity,
    tab_id_opt: Option<TabId>,
    color: Color,
    arrows_not_lines: bool,
) -> (Entity, Option<Entity>, Option<Entity>) {

    let mut edge_2d_entity = None;
    let mut edge_3d_entity = None;

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
    if let Some(tab_id) = tab_id_opt {
        commands.entity(vertex_2d_entity).insert(OwnedByTab::new(tab_id));
    }

    if let Some(parent_3d_entity) = parent_3d_entity_opt {
        // create 2d edge entity
        let shape_components = if arrows_not_lines {
            create_2d_edge_arrow(meshes, materials, Vec2::ZERO, Vec2::X, color)
        } else {
            create_2d_edge_line(meshes, materials, Vec2::ZERO, Vec2::X, color)
        };
        let new_entity = commands
            .spawn(shape_components)
            .insert(canvas_manager.layer_2d)
            .insert(Edge2d::new(vertex_2d_entity, parent_3d_entity))
            .id();
        if let Some(tab_id) = tab_id_opt {
            commands.entity(new_entity).insert(OwnedByTab::new(tab_id));
        }
        edge_2d_entity = Some(new_entity);

        // create 3d edge entity
        let shape_components = if arrows_not_lines {
            create_3d_edge_diamond(meshes, materials, Vec3::ZERO, Vec3::X, color)
        } else {
            create_3d_edge_line(meshes, materials, Vec3::ZERO, Vec3::X, color)
        };
        let new_entity = commands
            .spawn(shape_components)
            .insert(canvas_manager.layer_3d)
            .insert(Edge3d::new(vertex_3d_entity, parent_3d_entity))
            .id();
        if let Some(tab_id) = tab_id_opt {
            commands.entity(new_entity).insert(OwnedByTab::new(tab_id));
        }
        edge_3d_entity = Some(new_entity);
    } else {
        commands.entity(vertex_2d_entity).insert(VertexRootChild);
    }

    // info!(
    //     "created Vertex3d: `{:?}`, created 2d entity: {:?}, parent: {:?}",
    //     vertex_3d_entity, vertex_2d_entity, parent_entity_opt
    // );

    canvas_manager.register_3d_vertex(vertex_3d_entity, vertex_2d_entity);
    canvas_manager.recalculate_3d_view();

    (vertex_2d_entity, edge_2d_entity, edge_3d_entity)
}
