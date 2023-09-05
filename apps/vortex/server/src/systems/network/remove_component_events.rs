
use bevy_ecs::event::EventReader;
use bevy_log::info;

use naia_bevy_server::events::RemoveComponentEvents;

use vortex_proto::{
    components::{
        Edge3d, FileSystemChild, FileSystemRootChild,
        Vertex3d, VertexRoot,
    },
};

pub fn remove_component_events(mut event_reader: EventReader<RemoveComponentEvents>) {
    for events in event_reader.iter() {
        for (_user_key, _entity, _component) in events.read::<FileSystemRootChild>() {
            info!("removed FileSystemRootChild component from entity");
            // TODO!
        }
        for (_user_key, _entity, _component) in events.read::<FileSystemChild>() {
            info!("removed FileSystemChild component from entity");
            // TODO!
        }
        // on Vertex3D Remove Event
        for (_user_key, entity, _component) in events.read::<Vertex3d>() {
            info!("entity: `{:?}`, removed Vertex3d", entity);
        }
        // on Edge3d Remove Event
        for (_user_key, entity, _) in events.read::<Edge3d>() {
            info!("entity: `{:?}`, removed Edge3d", entity);
        }
        // on VertexRoot Remove Event
        for (_, entity, _) in events.read::<VertexRoot>() {
            panic!(
                "entity: `{:?}`, removed VertexRoot, how is this possible?",
                entity
            );
        }
    }
}