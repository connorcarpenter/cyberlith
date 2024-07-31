use std::{collections::HashMap, time::Duration};

use bevy_ecs::{
    entity::Entity,
    prelude::{Query, Resource},
    system::{Commands, ResMut},
};

use game_engine::{
    asset::{AssetHandle, UnitData},
    logging::info,
    math::Quat,
    naia::Replicate,
    render::components::{RenderLayers, Transform, Visibility},
    time::Instant,
    world::{
        components::{NextTilePosition, TileMovement},
        constants::MOVEMENT_SPEED,
        WorldClient,
    },
};
use game_engine::render::base::{CpuMaterial, CpuMesh};
use game_engine::storage::Storage;
use crate::{
    components::{AnimationState, Predicted},
    resources::{Global, OwnedEntity},
};
use crate::components::{RenderHelper, RenderPosition};

#[derive(Resource)]
pub(crate) struct PredictionEvents {
    records: HashMap<Entity, PredictionRecord>,
    last_pruned: Instant,
    prune_interval: Duration,
    record_ttl: Duration,
}

impl Default for PredictionEvents {
    fn default() -> Self {
        Self {
            records: HashMap::new(),
            last_pruned: Instant::now(),
            // how often to check for expired records
            prune_interval: Duration::from_secs(60),
            // how long to keep a record before removing
            record_ttl: Duration::from_secs(60),
        }
    }
}

impl PredictionEvents {
    // TODO: prune records regularily!

    // used as a system
    pub fn process(
        client: WorldClient,
        mut commands: Commands,
        mut global: ResMut<Global>,
        mut meshes: ResMut<Storage<CpuMesh>>,
        mut materials: ResMut<Storage<CpuMaterial>>,
        mut prediction_events: ResMut<PredictionEvents>,
        position_q: Query<&NextTilePosition>,
    ) {
        prediction_events.prune();

        let future_prediction_entities = prediction_events.recv_events();
        for (future_prediction_entity, unit_data_handle) in future_prediction_entities {
            info!(
                "future prediction entity is ready for processing: {:?}",
                future_prediction_entity
            );

            let client_tick = client.client_tick().unwrap();

            // Here we create a local copy of the Player entity, to use for client-side prediction
            let next_tile_position = position_q.get(future_prediction_entity).unwrap();

            let prediction_entity = commands.spawn_empty().id();

            commands
                .entity(prediction_entity)
                // Position stuff
                .insert(TileMovement::new_stopped(false, true, &next_tile_position))
                // Other rendering stuff
                .insert(RenderLayers::layer(0))
                .insert(Visibility::default())
                .insert(
                    Transform::default()
                        .with_rotation(Quat::from_rotation_z(f32::to_radians(90.0))),
                )
                .insert(AnimationState::new())
                .insert(RenderHelper::new(&mut meshes, &mut materials))
                .insert(RenderPosition::new())
                .insert(unit_data_handle.clone())
                // mark as predicted
                .insert(Predicted);

            info!(
                "from confirmed entity: {:?}, created prediction entity: {:?}",
                future_prediction_entity, prediction_entity,
            );
            global.owned_entity = Some(OwnedEntity::new(
                future_prediction_entity,
                prediction_entity,
            ));
        }
    }

    pub fn prune(&mut self) {
        let now = Instant::now();
        if self.last_pruned.elapsed(&now) > self.prune_interval {
            let mut to_remove = Vec::new();
            for (entity, record) in self.records.iter() {
                if record.last_update.elapsed(&now) > self.record_ttl {
                    to_remove.push(*entity);
                }
            }
            for entity in to_remove.iter() {
                self.records.remove(entity);
            }
            self.last_pruned = now;
        }
    }

    pub fn recv_events(&mut self) -> Vec<(Entity, AssetHandle<UnitData>)> {
        let mut results = Vec::new();
        for (entity, record) in self.records.iter() {
            if record.is_done() {
                let unit_data_handle = record.has_unit_asset_ref.as_ref().unwrap().clone();
                results.push((*entity, unit_data_handle));
            }
        }
        for (entity, _) in results.iter() {
            self.records.remove(entity);
        }
        results
    }

    pub fn read_insert_position_event(&mut self, now: &Instant, entity: &Entity) {
        info!(
            "received Inserted Position from World Server!  [ {:?} ]",
            entity,
        );
        if !self.records.contains_key(entity) {
            self.records.insert(*entity, PredictionRecord::new(now));
        }
        let record = self.records.get_mut(entity).unwrap();
        record.recv_position(now);
    }

    pub fn read_insert_unit_asset_ref_event(
        &mut self,
        now: &Instant,
        entity: &Entity,
        asset_handle: &AssetHandle<UnitData>,
    ) {
        info!(
            "received Inserted Unit Asset Ref from World Server!  [ {:?} ]",
            entity,
        );
        if !self.records.contains_key(entity) {
            self.records.insert(*entity, PredictionRecord::new(now));
        }
        let record = self.records.get_mut(entity).unwrap();
        record.recv_unit_asset_ref(now, asset_handle);
    }

    pub fn read_entity_assignment_event(&mut self, now: &Instant, entity: &Entity) {
        info!(
            "received Entity Assignment message from World Server!  [ {:?} ]",
            entity,
        );
        if !self.records.contains_key(entity) {
            self.records.insert(*entity, PredictionRecord::new(now));
        }
        let record = self.records.get_mut(entity).unwrap();
        record.recv_has_entity_assigment(now);
    }
}

struct PredictionRecord {
    last_update: Instant,
    has_position: Option<()>,
    has_unit_asset_ref: Option<AssetHandle<UnitData>>,
    has_entity_assigment: Option<()>,
}

impl PredictionRecord {
    pub fn new(now: &Instant) -> Self {
        Self {
            last_update: now.clone(),
            has_position: None,
            has_unit_asset_ref: None,
            has_entity_assigment: None,
        }
    }

    pub fn recv_position(&mut self, now: &Instant) {
        self.last_update = now.clone();
        self.has_position = Some(());
    }

    pub fn recv_unit_asset_ref(&mut self, now: &Instant, asset_handle: &AssetHandle<UnitData>) {
        self.last_update = now.clone();
        self.has_unit_asset_ref = Some(asset_handle.clone());
    }

    pub fn recv_has_entity_assigment(&mut self, now: &Instant) {
        self.last_update = now.clone();
        self.has_entity_assigment = Some(());
    }

    pub fn is_done(&self) -> bool {
        self.has_position.is_some()
            && self.has_unit_asset_ref.is_some()
            && self.has_entity_assigment.is_some()
    }
}
