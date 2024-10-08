use bevy_ecs::event::EventReader;
use logging::info;

use naia_bevy_server::events::RemoveComponentEvents;

use editor_proto::components::{
    AnimFrame, AnimRotation, Edge3d, Face3d, FileSystemChild, FileSystemRootChild, IconEdge,
    IconFace, IconFrame, IconVertex, PaletteColor, ShapeName, Vertex3d, VertexRoot,
};

pub fn remove_component_events(mut event_reader: EventReader<RemoveComponentEvents>) {
    for events in event_reader.read() {
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
        // on Face3d Remove Event
        for (_user_key, entity, _) in events.read::<Face3d>() {
            info!("entity: `{:?}`, removed Face3d", entity);
        }
        // on VertexRoot Remove Event
        for (_, entity, _) in events.read::<VertexRoot>() {
            panic!(
                "entity: `{:?}`, removed VertexRoot, how is this possible?",
                entity
            );
        }
        // on IconVertex Remove Event
        for (_user_key, entity, _component) in events.read::<IconVertex>() {
            info!("entity: `{:?}`, removed IconVertex", entity);
        }
        // on IconEdge Remove Event
        for (_user_key, entity, _) in events.read::<IconEdge>() {
            info!("entity: `{:?}`, removed IconEdge", entity);
        }
        // on IconFace Remove Event
        for (_user_key, entity, _) in events.read::<IconFace>() {
            info!("entity: `{:?}`, removed IconFace", entity);
        }
        // on IconFrame Remove Event
        for (_, entity, _) in events.read::<IconFrame>() {
            info!("entity: `{:?}`, removed IconFrame", entity);
        }

        // on ShapeName Remove Event
        for (_, entity, _) in events.read::<ShapeName>() {
            info!("entity: `{:?}`, removed ShapeName", entity);
        }
        // on AnimFrame Remove Event
        for (_, entity, _) in events.read::<AnimFrame>() {
            info!("entity: `{:?}`, removed AnimFrame", entity);
        }
        // on AnimRotation Remove Event
        for (_, entity, _) in events.read::<AnimRotation>() {
            info!("entity: `{:?}`, removed AnimRotation", entity);
        }
        // on PaletteColor Remove Event
        for (_, entity, _) in events.read::<PaletteColor>() {
            info!("entity: `{:?}`, removed PaletteColor", entity);
        }
    }
}
