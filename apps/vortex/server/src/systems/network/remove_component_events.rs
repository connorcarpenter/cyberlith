use bevy_ecs::{event::EventReader, system::ResMut};
use bevy_log::info;

use naia_bevy_server::events::RemoveComponentEvents;

use vortex_proto::components::{
    Edge3d, Face3d, FileSystemChild, FileSystemRootChild, ShapeName, Vertex3d, VertexRoot,
};

use crate::resources::ShapeManager;

pub fn remove_component_events(
    mut shape_manager: ResMut<ShapeManager>,
    mut event_reader: EventReader<RemoveComponentEvents>
) {
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
        // on ShapeName Remove Event
        for (user_key, entity, shape_name) in events.read::<ShapeName>() {
            info!("entity: `{:?}`, removed ShapeName", entity);

            let shape_name = shape_name.value.to_string();
            shape_manager.deregister_shape_name(&shape_name);
        }
    }
}
