use std::{collections::HashMap, any::TypeId};

use bevy_ecs::{event::EventReader, change_detection::ResMut, prelude::{Resource, Query}, entity::Entity, system::Commands};
use bevy_log::info;

use game_engine::{asset::{AssetCache, AssetId, AssetLoadedEvent, AssetMetadataStore, AnimationData, AssetHandle, AssetType, IconData, MeshFile, ModelData, PaletteData, SceneData, SkeletonData, SkinData, TypedAssetId},
                  world::{AssetEntry, AssetRef, WorldClient, Main, Alt1}};

use crate::app::systems::scene::WalkAnimation;

type AssetProcessorId = TypeId;

/// Stores asset data in RAM
#[derive(Resource)]
pub struct AssetRefProcessor {
    asset_processor_map: HashMap<AssetProcessorId, Box<dyn AssetDeferredProcessor>>,
    // entry entity -> (ref entity -> asset processor id)
    entry_waitlist: HashMap<Entity, HashMap<Entity, AssetProcessorId>>,
    // asset id -> (ref entity -> asset processor id)
    ref_waitlist: HashMap<AssetId, HashMap<Entity, AssetProcessorId>>,
}

impl AssetRefProcessor {
    pub fn new() -> Self {
        Self {
            asset_processor_map: HashMap::new(),
            entry_waitlist: HashMap::new(),
            ref_waitlist: HashMap::new(),
        }
    }

    // used by systems
    pub fn insert_asset_ref_events<T: AssetProcessor>(
        commands: &mut Commands,
        client: &WorldClient,
        asset_cache: &AssetCache,
        metadata_store: &mut AssetMetadataStore,
        asset_ref_processor: &mut AssetRefProcessor,
        asset_entry_q: &Query<&AssetEntry>,
        asset_ref_q: &Query<&AssetRef<T>>,
        entity: &Entity
    ) {
        let Ok(asset_ref) = asset_ref_q.get(*entity) else {
            panic!("Shouldn't happen");
        };
        let Some(asset_entry_entity) = asset_ref.asset_id_entity.get(client) else {
            panic!("Shouldn't happen");
        };
        if let Ok(asset_entry) = asset_entry_q.get(asset_entry_entity) {
            let asset_id = *asset_entry.asset_id;
            asset_ref_processor.handle_entity_added_asset_ref::<T>(commands, asset_cache, metadata_store, entity, &asset_id);
        } else {
            // asset entry entity has been replicated, but not the component just yet ...
            asset_ref_processor.handle_add_asset_entry_waitlist::<T>(entity, &asset_entry_entity);
        };
    }

    // used as a system
    pub fn handle_asset_loaded_events(
        mut commands: Commands,
        mut reader: EventReader<AssetLoadedEvent>,
        mut asset_ref_processer: ResMut<AssetRefProcessor>,
    ) {
        for event in reader.read() {
            info!("received Asset Loaded Event! (asset_id: {:?}, asset_type: {:?})", event.asset_id, event.asset_type);

            asset_ref_processer.process(&mut commands, event);
        }
    }

    fn process(&mut self, commands: &mut Commands, event: &AssetLoadedEvent) {

        let asset_id = event.asset_id;
        let asset_type = event.asset_type;

        // process any refs waiting for this asset
        if let Some(ref_waitlist_entry) = self.ref_waitlist.remove(&asset_id) {

            let typed_asset_id = TypedAssetId::new(asset_id, asset_type);

            for (entity, asset_processor_id) in ref_waitlist_entry {
                let asset_processor = self.asset_processor_map.remove(&asset_processor_id).unwrap();

                asset_processor.deferred_process(commands, &entity, &typed_asset_id);

                self.asset_processor_map.insert(asset_processor_id, asset_processor);
            }
        }
    }

    // entry entity is here, just not the component just yet ...
    pub fn handle_add_asset_entry_waitlist<T: AssetProcessor>(&mut self, ref_entity: &Entity, entry_entity: &Entity) {
        info!("entity ({:?}) received AssetRef from World Server! waiting on asset entry..", ref_entity);
        // initialize asset processor if needed
        let asset_processor_id = T::id();
        if !self.asset_processor_map.contains_key(&asset_processor_id) {
            self.asset_processor_map.insert(asset_processor_id, T::make_deferred_box());
        }

        if !self.entry_waitlist.contains_key(entry_entity) {
            self.entry_waitlist.insert(*entry_entity, HashMap::new());
        }
        let entry_waitlist_entry = self.entry_waitlist.get_mut(entry_entity).unwrap();
        entry_waitlist_entry.insert(*ref_entity, asset_processor_id);
    }

    pub fn handle_add_asset_entry(
        &mut self,
        commands: &mut Commands,
        metadata_store: &mut AssetMetadataStore,
        asset_cache: &AssetCache,
        entry_entity: &Entity,
        asset_id: &AssetId
    ) {
        info!("entity ({:?}) received AssetEntry from World Server! (asset_id: {:?})", entry_entity, asset_id);
        // initialize asset processor if needed
        if let Some(waitlist_entry) = self.entry_waitlist.remove(entry_entity) {
            for (ref_entity, asset_processor_id) in waitlist_entry {
                info!("Processing waiting AssetRef: {:?}", ref_entity);

                // mirror logic in handle_entity_added_asset_ref
                if asset_cache.has_asset(asset_id) {
                    // process asset ref
                    let asset_processor = self.asset_processor_map.remove(&asset_processor_id).unwrap();

                    let asset_type = metadata_store.get(asset_id).unwrap().asset_type();
                    let typed_asset_id = TypedAssetId::new(*asset_id, asset_type);
                    asset_processor.deferred_process(commands, &ref_entity, &typed_asset_id);

                    self.asset_processor_map.insert(asset_processor_id, asset_processor);
                } else {
                    // put ref into waitlist
                    info!("asset {:?} not yet loaded. adding to waitlist", asset_id);
                    if !self.ref_waitlist.contains_key(asset_id) {
                        self.ref_waitlist.insert(*asset_id, HashMap::new());
                    }
                    let ref_waitlist_entry = self.ref_waitlist.get_mut(asset_id).unwrap();
                    ref_waitlist_entry.insert(ref_entity, asset_processor_id);
                }
            }
        }
    }

    pub fn handle_entity_added_asset_ref<T: AssetProcessor>(
        &mut self,
        commands: &mut Commands,
        asset_cache: &AssetCache,
        metadata_store: &mut AssetMetadataStore,
        ref_entity: &Entity,
        asset_id: &AssetId
    ) {
        info!("entity ({:?}) received AssetRef from World Server! (asset_id: {:?})", ref_entity, asset_id);
        if asset_cache.has_asset(asset_id) {
            // process asset ref
            let asset_type = metadata_store.get(asset_id).unwrap().asset_type();
            let typed_asset_id = TypedAssetId::new(*asset_id, asset_type);
            T::process(commands, ref_entity, &typed_asset_id);
        } else {
            // initialize asset processor if needed
            let asset_processor_id = T::id();
            if !self.asset_processor_map.contains_key(&asset_processor_id) {
                self.asset_processor_map.insert(asset_processor_id, T::make_deferred_box());
            }

            // put ref into waitlist
            info!("asset {:?} not yet loaded. adding to waitlist", asset_id);
            if !self.ref_waitlist.contains_key(asset_id) {
                self.ref_waitlist.insert(*asset_id, HashMap::new());
            }
            let ref_waitlist_entry = self.ref_waitlist.get_mut(asset_id).unwrap();
            ref_waitlist_entry.insert(*ref_entity, asset_processor_id);
        }
    }
}

pub trait AssetProcessor: Send + Sync + 'static {
    fn id() -> AssetProcessorId;
    fn make_deferred_box() -> Box<dyn AssetDeferredProcessor>;
    fn process(commands: &mut Commands, entity: &Entity, typed_asset_id: &TypedAssetId);
}

impl AssetDeferredProcessor for Main {
    fn deferred_process(&self, commands: &mut Commands, entity: &Entity, typed_asset_id: &TypedAssetId) {
        Self::process(commands, entity, typed_asset_id);
    }
}

impl AssetProcessor for Main {
    fn id() -> AssetProcessorId {
        TypeId::of::<Self>()
    }
    fn make_deferred_box() -> Box<dyn AssetDeferredProcessor> {
        Box::new(Self)
    }
    fn process(commands: &mut Commands, entity: &Entity, typed_asset_id: &TypedAssetId) {
        info!("processing for entity: {:?} = inserting AssetRef<Main>(asset_id: {:?}) ", entity, typed_asset_id.get_id());

        let asset_type = typed_asset_id.get_type();
        let asset_id = typed_asset_id.get_id();
        match asset_type {
            AssetType::Skeleton => process_type::<SkeletonData>(commands, entity, &asset_id),
            AssetType::Mesh => process_type::<MeshFile>(commands, entity, &asset_id),
            AssetType::Palette => process_type::<PaletteData>(commands, entity, &asset_id),
            AssetType::Animation => process_type::<AnimationData>(commands, entity, &asset_id),
            AssetType::Icon => process_type::<IconData>(commands, entity, &asset_id),
            AssetType::Skin => process_type::<SkinData>(commands, entity, &asset_id),
            AssetType::Model => process_type::<ModelData>(commands, entity, &asset_id),
            AssetType::Scene => process_type::<SceneData>(commands, entity, &asset_id),
        }
    }
}

impl AssetDeferredProcessor for Alt1 {
    fn deferred_process(&self, commands: &mut Commands, entity: &Entity, typed_asset_id: &TypedAssetId) {
        Self::process(commands, entity, typed_asset_id);
    }
}

impl AssetProcessor for Alt1 {
    fn id() -> AssetProcessorId {
        TypeId::of::<Self>()
    }
    fn make_deferred_box() -> Box<dyn AssetDeferredProcessor> {
        Box::new(Self)
    }
    fn process(commands: &mut Commands, entity: &Entity, typed_asset_id: &TypedAssetId) {
        info!("processing for entity: {:?} = inserting AssetRef<Alt1>(asset_id: {:?}) ", entity, typed_asset_id.get_id());

        let asset_type = typed_asset_id.get_type();
        let asset_id = typed_asset_id.get_id();
        match asset_type {
            AssetType::Animation => {
                let walk_anim = WalkAnimation::new(AssetHandle::<AnimationData>::new(asset_id));
                commands.entity(*entity).insert(walk_anim);
            }
            _ => { panic!("unsupported asset type: {:?}", asset_type); }
        }
    }
}

fn process_type<T: Send + Sync + 'static>(commands: &mut Commands, entity: &Entity, asset_id: &AssetId) {
    commands.entity(*entity).insert(AssetHandle::<T>::new(*asset_id));
}

pub trait AssetDeferredProcessor: Send + Sync + 'static {
    fn deferred_process(&self, commands: &mut Commands, entity: &Entity, typed_asset_id: &TypedAssetId);
}