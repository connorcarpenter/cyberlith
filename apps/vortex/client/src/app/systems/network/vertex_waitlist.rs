use std::collections::HashMap;

use bevy_ecs::{entity::Entity, system::Commands};

use naia_bevy_client::{Client, CommandsExt, ReplicationConfig};

use math::{Vec2, Vec3};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    Assets,
};
use vortex_proto::{
    components::{Edge3d, OwnedByTab},
    types::TabId,
};

use crate::app::{
    components::{Edge2dLocal, Vertex2d},
    resources::{camera_manager::CameraManager, vertex_manager::VertexManager},
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

    pub(crate) fn decompose(self) -> (Option<Entity>, TabId) {
        return (
            self.parent.unwrap(),
            self.tab_id.unwrap(),
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

    pub fn vertex_process_insert(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        entity: &Entity,
        insert: VertexWaitlistInsert,
    ) {
        if !self.contains_key(&entity) {
            self.insert_incomplete(*entity, VertexWaitlistEntry::new());
        }
        let entry = self.get_mut(&entity).unwrap();

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
        }

        if entry.is_ready() {
            let entry = self.remove(entity).unwrap();

            if entry.has_parent() {
                let parent_entity = entry.get_parent().unwrap();
                if !vertex_manager.has_vertex_entity_3d(&parent_entity) {
                    // need to put in parent waitlist
                    // info!(
                    //     "entity {:?} requires parent {:?}. putting in parent waitlist",
                    //     entity,
                    //     parent_entity
                    // );
                    self.insert_parent_waiting(parent_entity, *entity, entry);
                    return;
                }
            }
            self.vertex_process_insert_complete(
                commands,
                client,
                meshes,
                materials,
                camera_manager,
                vertex_manager,
                *entity,
                entry,
            );
        }
    }

    fn vertex_process_insert_complete(
        &mut self,
        commands: &mut Commands,
        client: &mut Client,
        meshes: &mut Assets<CpuMesh>,
        materials: &mut Assets<CpuMaterial>,
        camera_manager: &mut CameraManager,
        vertex_manager: &mut VertexManager,
        entity: Entity,
        entry: VertexWaitlistEntry,
    ) {
        // info!("processing complete vertex {:?}", entity);

        let (parent_3d_entity_opt, tab_id) = entry.decompose();

        let color = match parent_3d_entity_opt {
            Some(_) => Vertex2d::CHILD_COLOR,
            None => Vertex2d::ROOT_COLOR,
        };

        let _new_vertex_2d_entity = vertex_manager.vertex_3d_postprocess(
            commands,
            meshes,
            materials,
            camera_manager,
            entity,
            parent_3d_entity_opt.is_none(),
            Some(tab_id),
            color,
        );

        // if the waitlist has any children entities of this one, process them
        if let Some(child_entries) = self.on_vertex_complete(entity) {
            for (child_entity, child_entry) in child_entries {
                // info!("entity {:?} was waiting on parent {:?}. processing!", child_entity, entity);
                self.vertex_process_insert_complete(
                    commands,
                    client,
                    meshes,
                    materials,
                    camera_manager,
                    vertex_manager,
                    child_entity,
                    child_entry,
                );
            }
        }

        camera_manager.recalculate_3d_view();
        vertex_manager.recalculate_vertices();
    }

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
