use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Commands};

use math::{Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    Assets,
};
use vortex_proto::{
    components::{OwnedByTab, VertexTypeValue},
    types::TabId,
};

use crate::app::{
    components::{Edge2dLocal, Edge3dLocal, Vertex2d},
    resources::{camera_manager::CameraManager, vertex_manager::VertexManager},
    shapes::{
        create_2d_edge_arrow, create_2d_edge_line, create_3d_edge_diamond, create_3d_edge_line,
    },
};

pub enum VertexWaitlistInsert {
    Position,
    Parent(Option<Entity>),
    OwnedByTab(TabId),
    Type(VertexTypeValue),
}

pub struct VertexWaitlistEntry {
    has_pos: bool,
    parent: Option<Option<Entity>>,
    tab_id: Option<TabId>,
    vertex_type: Option<VertexTypeValue>,
}

impl VertexWaitlistEntry {
    fn new() -> Self {
        Self {
            has_pos: false,
            parent: None,
            tab_id: None,
            vertex_type: None,
        }
    }

    fn is_ready(&self) -> bool {
        self.has_pos && self.parent.is_some() && self.tab_id.is_some() && self.vertex_type.is_some()
    }

    fn set_parent(&mut self, parent: Option<Entity>) {
        self.parent = Some(parent);
    }

    fn get_parent(&self) -> Option<Entity> {
        self.parent.unwrap()
    }

    fn has_parent(&self) -> bool {
        if let Some(parent_opt) = &self.parent {
            return parent_opt.is_some();
        }
        return false;
    }

    fn set_position(&mut self) {
        self.has_pos = true;
    }

    fn set_tab_id(&mut self, tab_id: TabId) {
        self.tab_id = Some(tab_id);
    }

    fn set_type(&mut self, value: VertexTypeValue) {
        self.vertex_type = Some(value);
    }

    pub(crate) fn decompose(self) -> (Option<Entity>, TabId, VertexTypeValue) {
        return (
            self.parent.unwrap(),
            self.tab_id.unwrap(),
            self.vertex_type.unwrap(),
        );
    }
}

pub struct VertexWaitlist {
    incomplete_entries: HashMap<Entity, VertexWaitlistEntry>,
    waiting_on_parent: HashMap<Entity, Vec<(Entity, VertexWaitlistEntry)>>,
}

impl Default for VertexWaitlist {
    fn default() -> Self {
        Self {
            incomplete_entries: HashMap::new(),
            waiting_on_parent: HashMap::new(),
        }
    }
}

impl VertexWaitlist {
    fn contains_key(&self, entity: &Entity) -> bool {
        self.incomplete_entries.contains_key(entity)
    }

    fn insert_incomplete(&mut self, entity: Entity, entry: VertexWaitlistEntry) {
        self.incomplete_entries.insert(entity, entry);
    }

    fn get_mut(&mut self, entity: &Entity) -> Option<&mut VertexWaitlistEntry> {
        self.incomplete_entries.get_mut(entity)
    }

    fn remove(&mut self, entity: &Entity) -> Option<VertexWaitlistEntry> {
        self.incomplete_entries.remove(entity)
    }

    fn insert_parent_waiting(
        &mut self,
        parent_entity: Entity,
        child_entity: Entity,
        child_entry: VertexWaitlistEntry,
    ) {
        if !self.waiting_on_parent.contains_key(&parent_entity) {
            self.waiting_on_parent.insert(parent_entity, Vec::new());
        }
        let entries = self.waiting_on_parent.get_mut(&parent_entity).unwrap();
        entries.push((child_entity, child_entry));
    }

    fn on_vertex_complete(&mut self, entity: Entity) -> Option<Vec<(Entity, VertexWaitlistEntry)>> {
        self.waiting_on_parent.remove(&entity)
    }
}

pub fn vertex_process_insert(
    commands: &mut Commands,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    camera_manager: &mut CameraManager,
    vertex_manager: &mut VertexManager,
    vertex_waitlist: &mut VertexWaitlist,
    entity: &Entity,
    insert: VertexWaitlistInsert,
) {
    if !vertex_waitlist.contains_key(&entity) {
        vertex_waitlist.insert_incomplete(*entity, VertexWaitlistEntry::new());
    }
    let entry = vertex_waitlist.get_mut(&entity).unwrap();

    match insert {
        VertexWaitlistInsert::Position => {
            entry.set_position();
        }
        VertexWaitlistInsert::Parent(parent) => {
            entry.set_parent(parent);
        }
        VertexWaitlistInsert::OwnedByTab(tab_id) => {
            entry.set_tab_id(tab_id);
        }
        VertexWaitlistInsert::Type(value) => {
            entry.set_type(value);
        }
    }

    if entry.is_ready() {
        let entry = vertex_waitlist.remove(entity).unwrap();

        if entry.has_parent() {
            let parent_entity = entry.get_parent().unwrap();
            if !vertex_manager.has_vertex_entity_3d(&parent_entity) {
                // need to put in parent waitlist
                // info!(
                //     "entity {:?} requires parent {:?}. putting in parent waitlist",
                //     entity,
                //     parent_entity
                // );
                vertex_waitlist.insert_parent_waiting(parent_entity, *entity, entry);
                return;
            }
        }
        vertex_process_insert_complete(
            commands,
            meshes,
            materials,
            camera_manager,
            vertex_manager,
            vertex_waitlist,
            *entity,
            entry,
        );
    }
}

fn vertex_process_insert_complete(
    commands: &mut Commands,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    camera_manager: &mut CameraManager,
    vertex_manager: &mut VertexManager,
    vertex_waitlist: &mut VertexWaitlist,
    entity: Entity,
    entry: VertexWaitlistEntry,
) {
    // info!("processing complete vertex {:?}", entity);

    let (parent_3d_entity_opt, tab_id, type_value) = entry.decompose();

    let color = match parent_3d_entity_opt {
        Some(_) => Vertex2d::CHILD_COLOR,
        None => Vertex2d::ROOT_COLOR,
    };

    let new_vertex_2d_entity = vertex_manager.vertex_3d_postprocess(
        commands,
        meshes,
        materials,
        camera_manager,
        entity,
        parent_3d_entity_opt.is_none(),
        Some(tab_id),
        color,
    );

    // if vertex has parent, create an edge
    if let Some(parent_3d_entity) = parent_3d_entity_opt {
        let Some(parent_2d_entity) = vertex_manager
            .vertex_entity_3d_to_2d(&parent_3d_entity) else {
            panic!("Parent 3d entity {:?} has no 2d entity", parent_3d_entity);
        };
        edge_3d_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            entity,
            new_vertex_2d_entity,
            parent_3d_entity,
            *parent_2d_entity,
            Some(tab_id),
            Vertex2d::CHILD_COLOR,
            type_value == VertexTypeValue::Skel,
        );
    }

    // if the waitlist has any children entities of this one, process them
    if let Some(child_entries) = vertex_waitlist.on_vertex_complete(entity) {
        for (child_entity, child_entry) in child_entries {
            // info!("entity {:?} was waiting on parent {:?}. processing!", child_entity, entity);
            vertex_process_insert_complete(
                commands,
                meshes,
                materials,
                camera_manager,
                vertex_manager,
                vertex_waitlist,
                child_entity,
                child_entry,
            );
        }
    }

    camera_manager.recalculate_3d_view();
    vertex_manager.recalculate_vertices();
}

pub fn edge_3d_postprocess(
    commands: &mut Commands,
    meshes: &mut Assets<CpuMesh>,
    materials: &mut Assets<CpuMaterial>,
    camera_manager: &mut CameraManager,
    vertex_a_3d_entity: Entity,
    vertex_a_2d_entity: Entity,
    vertex_b_3d_entity: Entity,
    vertex_b_2d_entity: Entity,
    tab_id_opt: Option<TabId>,
    color: Color,
    arrows_not_lines: bool,
) -> (Entity, Entity) {
    // create 2d edge entity
    let shape_components = if arrows_not_lines {
        create_2d_edge_arrow(meshes, materials, Vec2::ZERO, Vec2::X, color)
    } else {
        create_2d_edge_line(meshes, materials, Vec2::ZERO, Vec2::X, color)
    };
    let new_entity = commands
        .spawn(shape_components)
        .insert(camera_manager.layer_2d)
        .insert(Edge2dLocal::new(vertex_a_2d_entity, vertex_b_2d_entity))
        .id();
    if let Some(tab_id) = tab_id_opt {
        commands.entity(new_entity).insert(OwnedByTab::new(tab_id));
    }
    let edge_2d_entity = new_entity;

    // create 3d edge entity
    let shape_components = if arrows_not_lines {
        create_3d_edge_diamond(meshes, materials, Vec3::ZERO, Vec3::X, color)
    } else {
        create_3d_edge_line(meshes, materials, Vec3::ZERO, Vec3::X, color)
    };
    let new_entity = commands
        .spawn(shape_components)
        .insert(camera_manager.layer_3d)
        .insert(Edge3dLocal::new(vertex_a_3d_entity, vertex_b_3d_entity))
        .id();
    if let Some(tab_id) = tab_id_opt {
        commands.entity(new_entity).insert(OwnedByTab::new(tab_id));
    }
    let edge_3d_entity = new_entity;

    camera_manager.recalculate_3d_view();

    (edge_2d_entity, edge_3d_entity)
}
