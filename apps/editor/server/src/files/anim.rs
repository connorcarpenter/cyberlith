use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, ResMut, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{BitReader, CommandsExt, ReplicationConfig, Server};

use asset_io::AnimAction;

use editor_proto::{
    components::{AnimFrame, AnimRotation, FileExtension, Transition},
    resources::FileKey,
    SerdeQuat,
};

use crate::{
    files::{
        add_file_dependency, convert_from_quat, convert_from_transition, convert_into_quat_map,
        convert_into_transition, FileWriter,
    },
    resources::{AnimationManager, ContentEntityData, Project},
};

// Writer
pub struct AnimWriter;

impl AnimWriter {
    fn world_to_actions(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Vec<AnimAction> {
        let working_file_entries = project.working_file_entries();

        let mut skel_dependency_key_opt = None;
        let mut shape_names: Vec<String> = Vec::new();
        let mut shape_map: HashMap<String, u16> = HashMap::new();

        let mut biggest_order_opt: Option<u8> = None;
        //////////////////// order, frame_entity, duration_5ms
        let mut frame_map: HashMap<u8, (Entity, Transition)> = HashMap::new();
        let mut frame_poses_map: HashMap<Entity, HashMap<u16, SerdeQuat>> = HashMap::new();

        for (content_entity, content_data) in content_entities {
            match content_data {
                ContentEntityData::Dependency(dependency_key) => {
                    let dependency_value = working_file_entries.get(dependency_key).unwrap();
                    if dependency_value.extension().unwrap() != FileExtension::Skel {
                        panic!("anim file should depend on a single .skel file");
                    }
                    skel_dependency_key_opt = Some(dependency_key);
                }
                ContentEntityData::Frame => {
                    let mut system_state: SystemState<Query<&AnimFrame>> = SystemState::new(world);
                    let frame_q = system_state.get_mut(world);

                    let Ok(frame) = frame_q.get(*content_entity) else {
                        panic!("Error getting frame component");
                    };

                    let frame_order = frame.get_order();

                    info!("processing frame: {}", frame_order);

                    // update biggest order
                    if let Some(biggest_order) = biggest_order_opt {
                        if frame_order > biggest_order {
                            biggest_order_opt = Some(frame_order);
                        }
                    } else {
                        biggest_order_opt = Some(frame_order);
                    }

                    if frame_map.contains_key(&frame_order) {
                        panic!("anim file should not have duplicate frame orders");
                    }
                    frame_map.insert(frame_order, (*content_entity, (*frame.transition).clone()));
                }
                ContentEntityData::Rotation => {
                    let mut system_state: SystemState<(Server, Query<&AnimRotation>)> =
                        SystemState::new(world);
                    let (server, rot_q) = system_state.get_mut(world);

                    let Ok(rotation) = rot_q.get(*content_entity) else {
                        panic!("Error getting rotation component");
                    };

                    // get shape name
                    let name = (*rotation.vertex_name).clone();
                    let shape_index: u16 = if !shape_map.contains_key(&name) {
                        let shape_index = shape_names.len() as u16;
                        shape_map.insert(name.clone(), shape_index);
                        shape_names.push(name);
                        shape_index
                    } else {
                        *shape_map.get(&name).unwrap()
                    };

                    // get & add to frame
                    let frame_entity: Entity = rotation.frame_entity.get(&server).unwrap();
                    if !frame_poses_map.contains_key(&frame_entity) {
                        frame_poses_map.insert(frame_entity, HashMap::new());
                    }
                    let poses_map = frame_poses_map.get_mut(&frame_entity).unwrap();
                    poses_map.insert(shape_index, rotation.get_rotation_serde());
                }
                _ => {
                    panic!("animation should not have this content entity type");
                }
            }
        }

        let mut actions = Vec::new();

        // Write Skel Dependency
        if let Some(dependency_key) = skel_dependency_key_opt {
            info!("writing dependency: {}", dependency_key.full_path());
            actions.push(AnimAction::SkelFile(
                dependency_key.path().to_string(),
                dependency_key.name().to_string(),
            ));
        }

        // Write Shape Names
        for shape_name in shape_names {
            info!("writing shape name: {}", shape_name);
            actions.push(AnimAction::ShapeIndex(shape_name));
        }

        // Write Frames
        if let Some(biggest_order) = biggest_order_opt {
            for order in 0..=biggest_order {
                let Some((frame_entity, transition)) = frame_map.remove(&order) else {
                    panic!("anim file should not have any gaps in frame orders");
                };
                let poses = if let Some(poses) = frame_poses_map.remove(&frame_entity) {
                    poses
                } else {
                    HashMap::new()
                };
                info!("push frame action: {}", order);
                actions.push(AnimAction::Frame(
                    convert_into_quat_map(poses),
                    convert_into_transition(transition),
                ));
            }
        }

        actions
    }
}

impl FileWriter for AnimWriter {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        content_entities: &HashMap<Entity, ContentEntityData>,
    ) -> Box<[u8]> {
        let actions = self.world_to_actions(world, project, content_entities);
        AnimAction::write(actions)
    }

    fn write_new_default(&self) -> Box<[u8]> {
        info!("anim write new default");
        let mut actions = Vec::new();

        actions.push(AnimAction::Frame(
            HashMap::new(),
            convert_into_transition(Transition::new(100)),
        ));

        AnimAction::write(actions)
    }
}

// Reader
pub struct AnimReader;

impl AnimReader {
    fn actions_to_world(
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        actions: Vec<AnimAction>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut output = HashMap::new();
        let mut shape_name_index = 0;
        let mut shape_name_map = HashMap::new();
        let mut frame_index = 0;

        let mut system_state: SystemState<(Commands, Server, ResMut<AnimationManager>)> =
            SystemState::new(world);
        let (mut commands, mut server, mut animation_manager) = system_state.get_mut(world);

        for action in actions {
            match action {
                AnimAction::SkelFile(skel_path, skel_file_name) => {
                    let (new_entity, _, new_file_key) = add_file_dependency(
                        project,
                        file_key,
                        file_entity,
                        &mut commands,
                        &mut server,
                        FileExtension::Skel,
                        &skel_path,
                        &skel_file_name,
                    );
                    output.insert(new_entity, ContentEntityData::new_dependency(new_file_key));
                }
                AnimAction::ShapeIndex(shape_name) => {
                    shape_name_map.insert(shape_name_index, shape_name);
                    shape_name_index += 1;
                }
                AnimAction::Frame(poses, transition) => {
                    info!("read frame action!");

                    let mut component =
                        AnimFrame::new(frame_index, convert_from_transition(transition));
                    component.file_entity.set(&server, file_entity);
                    let frame_entity = commands
                        .spawn_empty()
                        .enable_replication(&mut server)
                        .configure_replication(ReplicationConfig::Delegated)
                        .insert(component)
                        .id();

                    output.insert(frame_entity, ContentEntityData::new_frame());

                    animation_manager.on_create_frame(
                        &file_entity,
                        &frame_entity,
                        frame_index as usize,
                        None,
                    );

                    for (shape_index, rotation) in poses {
                        let shape_name = shape_name_map.get(&shape_index).unwrap();

                        let mut component =
                            AnimRotation::new(shape_name.clone(), convert_from_quat(rotation));
                        component.frame_entity.set(&server, &frame_entity);

                        let rotation_entity = commands
                            .spawn_empty()
                            .enable_replication(&mut server)
                            .configure_replication(ReplicationConfig::Delegated)
                            .insert(component)
                            .id();

                        output.insert(rotation_entity, ContentEntityData::new_rotation());

                        animation_manager.on_create_rotation(frame_entity, rotation_entity);
                    }

                    frame_index += 1;
                }
            }
        }

        system_state.apply(world);

        output
    }

    pub fn read(
        &self,
        world: &mut World,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        bytes: &Box<[u8]>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut bit_reader = BitReader::new(bytes);

        let Ok(actions) = AnimAction::read(&mut bit_reader) else {
            panic!("Error reading .anim file");
        };

        let result = Self::actions_to_world(world, project, file_key, file_entity, actions);

        result
    }
}
