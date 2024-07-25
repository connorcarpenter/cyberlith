use std::collections::HashMap;

use bevy_ecs::{system::{Commands, ResMut}, entity::Entity, prelude::{Resource, Query}};

use game_engine::{
    logging::info,
    world::components::Position,
    asset::{AssetHandle, UnitData},
    naia::Replicate,
    render::components::{RenderLayers, Transform, Visibility},
};

use crate::{resources::{Global, OwnedEntity}, components::{AnimationState, Interp, Predicted}};

#[derive(Resource)]
pub(crate) struct PredictionEvents {
    records: HashMap<Entity, PredictionRecord>,
}

impl Default for PredictionEvents {
    fn default() -> Self {
        Self {
            records: HashMap::new(),
        }
    }
}

impl PredictionEvents {

    // TODO: prune records regularily!

    // used as a system
    pub fn process(
        mut commands: Commands,
        mut global: ResMut<Global>,
        mut prediction_events: ResMut<PredictionEvents>,
        position_q: Query<&Position>,
        unit_q: Query<&AssetHandle<UnitData>>,
    ) {
        let future_prediction_entities = prediction_events.recv_events();
        for future_prediction_entity in future_prediction_entities {
            info!("future prediction entity is ready for processing: {:?}", future_prediction_entity);

            // Here we create a local copy of the Player entity, to use for client-side prediction
            let position = position_q.get(future_prediction_entity).unwrap();
            let unit_data_handle = unit_q.get(future_prediction_entity).unwrap();

            let prediction_entity = commands
                .spawn_empty()
                .id();
            let mut prediction_position = Position::new(*position.x, *position.y);
            prediction_position.localize();
            commands
                .entity(prediction_entity)
                .insert(prediction_position)
                .insert(RenderLayers::layer(0))
                .insert(Visibility::default())
                .insert(Transform::default())
                .insert(AnimationState::new())
                .insert(unit_data_handle.clone())
                // insert interpolation component
                .insert(Interp::new(*position.x, *position.y))
                // mark as predicted
                .insert(Predicted);

            info!(
                "from confirmed entity: {:?}, created prediction entity: {:?}",
                future_prediction_entity,
                prediction_entity,
            );
            global.owned_entity = Some(OwnedEntity::new(future_prediction_entity, prediction_entity));
        }
    }

    pub fn recv_events(&mut self) -> Vec<Entity> {
        let mut results = Vec::new();
        for (entity, record) in self.records.iter() {
            if record.is_done() {
                results.push(*entity);
            }
        }
        for entity in results.iter() {
            self.records.remove(entity);
        }
        results
    }

    pub fn read_insert_position_event(
        &mut self,
        entity: &Entity,
    ) {
        info!(
            "received Inserted Position from World Server!  [ {:?} ]",
            entity,
        );
        if !self.records.contains_key(entity) {
            self.records.insert(*entity, PredictionRecord::new());
        }
        let record = self.records.get_mut(entity).unwrap();
        record.recv_position();
    }

    pub fn read_insert_unit_asset_ref_event(
        &mut self,
        entity: &Entity,
    ) {
        info!(
            "received Inserted Unit Asset Ref from World Server!  [ {:?} ]",
            entity,
        );
        if !self.records.contains_key(entity) {
            self.records.insert(*entity, PredictionRecord::new());
        }
        let record = self.records.get_mut(entity).unwrap();
        record.recv_unit_asset_ref();
    }

    pub fn read_entity_assignment_event(
        &mut self,
        entity: &Entity,
    ) {
        info!(
            "received Entity Assignment message from World Server!  [ {:?} ]",
            entity,
        );
        if !self.records.contains_key(entity) {
            self.records.insert(*entity, PredictionRecord::new());
        }
        let record = self.records.get_mut(entity).unwrap();
        record.recv_has_entity_assigment();
    }
}

struct PredictionRecord {
    has_position: Option<()>,
    has_unit_asset_ref: Option<()>,
    has_entity_assigment: Option<()>,
}

impl PredictionRecord {
    pub fn new() -> Self {
        Self {
            has_position: None,
            has_unit_asset_ref: None,
            has_entity_assigment: None,
        }
    }

    pub fn recv_position(&mut self) {
        self.has_position = Some(());
    }

    pub fn recv_unit_asset_ref(&mut self) {
        self.has_unit_asset_ref = Some(());
    }

    pub fn recv_has_entity_assigment(&mut self) {
        self.has_entity_assigment = Some(());
    }

    pub fn is_done(&self) -> bool {
        self.has_position.is_some() && self.has_unit_asset_ref.is_some() && self.has_entity_assigment.is_some()
    }
}
