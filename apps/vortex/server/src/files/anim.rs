use std::collections::HashMap;

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, World},
    system::{Query, SystemState},
};
use bevy_log::info;

use naia_bevy_server::{
    BitReader, CommandsExt, FileBitWriter, ReplicationConfig, Serde, SerdeErr, Server,
    UnsignedVariableInteger,
};

use vortex_proto::{
    components::{EntryKind, FileDependency, FileExtension, AnimFrame, AnimRotation, ShapeName, Transition},
    resources::FileKey,
    SerdeQuat,
};

use crate::{
    files::{FileReadOutput, FileReader, FileWriter},
    resources::{ContentEntityData, Project},
};

// Actions
enum AnimAction {
    // path, file_name
    SkelFile(String, String),
    // shape name -> shape_index
    ShapeIndex(String),
    // shape_index -> rotation
    Frame(HashMap<u16, SerdeQuat>, Transition),
}

#[derive(Serde, Clone, PartialEq)]
enum AnimActionType {
    SkelFile,
    ShapeIndex,
    Frame,
    None,
}

// Writer
pub struct AnimWriter;

impl AnimWriter {
    fn world_to_actions(
        &self,
        world: &mut World,
        project: &Project,
        content_entities_opt: &Option<HashMap<Entity, ContentEntityData>>,
    ) -> Vec<AnimAction> {

        let working_file_entries = project.working_file_entries();

        let mut skel_dependency_key_opt = None;
        let mut shape_names: Vec<String> = Vec::new();
        let mut shape_map: HashMap<String, u16> = HashMap::new();

        let mut biggest_order_opt: Option<u8> = None;
        //////////////////// order, frame_entity, duration_5ms
        let mut frame_map: HashMap<u8, (Entity, Transition)> = HashMap::new();
        let mut frame_poses_map: HashMap<Entity, HashMap<u16, SerdeQuat>> = HashMap::new();

        let content_entities = content_entities_opt.as_ref().unwrap();
        for (content_entity, content_data) in content_entities {
            match content_data {
                ContentEntityData::Shape(_) => {
                    panic!("animation should not have any shape entity in it");
                }
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
                    let mut system_state: SystemState<(Server, Query<&AnimRotation>, Query<&ShapeName>)> = SystemState::new(world);
                    let (server, rot_q, name_q) = system_state.get_mut(world);

                    let Ok(rotation) = rot_q.get(*content_entity) else {
                        panic!("Error getting rotation component");
                    };

                    // get shape name
                    let vertex_3d_entity: Entity = rotation.vertex_3d_entity.get(&server).unwrap();
                    let Ok(name) = name_q.get(vertex_3d_entity) else {
                        panic!("Error getting shape name component");
                    };
                    let name = (*name.value).clone();
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
            }
        }

        let mut actions = Vec::new();

        // Write Skel Dependency
        let Some(dependency_key) = skel_dependency_key_opt else {
            panic!("anim file should depend on a single .skel file");
        };

        info!("writing dependency: {}", dependency_key.full_path());
        actions.push(AnimAction::SkelFile(
            dependency_key.path().to_string(),
            dependency_key.name().to_string(),
        ));

        // Write Shape Names
        for shape_name in shape_names {
            info!("writing shape name: {}", shape_name);
            actions.push(AnimAction::ShapeIndex(shape_name));
        }

        // Write Frames
        let biggest_order = biggest_order_opt.unwrap();
        for order in 0..=biggest_order {

            let Some((frame_entity, transition)) = frame_map.remove(&order) else {
                panic!("anim file should not have any gaps in frame orders");
            };
            let Some(poses) = frame_poses_map.remove(&frame_entity) else {
                panic!("anim file should not have any gaps in frame orders");
            };
            info!("writing frame: {}", order);
            actions.push(AnimAction::Frame(poses, transition));
        }

        actions
    }

    fn write_from_actions(&self, actions: Vec<AnimAction>) -> Box<[u8]> {
        let mut bit_writer = FileBitWriter::new();

        for action in actions {
            match action {
                AnimAction::SkelFile(path, file_name) => {
                    AnimActionType::SkelFile.ser(&mut bit_writer);
                    path.ser(&mut bit_writer);
                    file_name.ser(&mut bit_writer);
                }
                AnimAction::ShapeIndex(name) => {
                    AnimActionType::ShapeIndex.ser(&mut bit_writer);
                    name.ser(&mut bit_writer);
                }
                AnimAction::Frame(poses, transition) => {
                    AnimActionType::Frame.ser(&mut bit_writer);
                    transition.ser(&mut bit_writer);
                    for (shape_index, pose) in poses {
                        // continue bit
                        true.ser(&mut bit_writer);

                        UnsignedVariableInteger::<5>::from(shape_index).ser(&mut bit_writer);
                        pose.ser(&mut bit_writer);
                    }
                    // continue bit
                    false.ser(&mut bit_writer);
                }
            }
        }

        // continue bit
        AnimActionType::None.ser(&mut bit_writer);

        bit_writer.to_bytes()
    }
}

impl FileWriter for AnimWriter {
    fn write(
        &self,
        world: &mut World,
        project: &Project,
        content_entities_opt: &Option<HashMap<Entity, ContentEntityData>>,
    ) -> Box<[u8]> {
        let actions = self.world_to_actions(world, project, content_entities_opt);
        self.write_from_actions(actions)
    }

    fn write_new_default(&self) -> Box<[u8]> {
        self.write_from_actions(Vec::new())
    }
}

// Reader
pub struct AnimReader;

impl AnimReader {
    fn read_to_actions(bit_reader: &mut BitReader) -> Result<Vec<AnimAction>, SerdeErr> {
        let mut actions = Vec::new();

        loop {
            let action_type = AnimActionType::de(bit_reader)?;
            match action_type {
                AnimActionType::SkelFile => {
                    let path = String::de(bit_reader)?;
                    let file_name = String::de(bit_reader)?;
                    actions.push(AnimAction::SkelFile(path, file_name));
                }
                AnimActionType::ShapeIndex => {
                    let name = String::de(bit_reader)?;
                    actions.push(AnimAction::ShapeIndex(name));
                }
                AnimActionType::Frame => {
                    let transition = Transition::de(bit_reader)?;
                    let mut poses = HashMap::new();
                    loop {
                        let continue_bit = bool::de(bit_reader)?;
                        if !continue_bit {
                            break;
                        }

                        let shape_index: u16 = UnsignedVariableInteger::<5>::de(bit_reader)?.to();
                        let pose = SerdeQuat::de(bit_reader)?;
                        poses.insert(shape_index, pose);
                    }
                    actions.push(AnimAction::Frame(poses, transition));
                }
                AnimActionType::None => {
                    break;
                }
            }
        }

        Ok(actions)
    }

    fn actions_to_world(
        _commands: &mut Commands,
        _server: &mut Server,
        actions: Vec<AnimAction>,
    ) -> Result<FileReadOutput, SerdeErr> {
        let mut skel_path = None;

        for action in actions {
            match action {
                AnimAction::SkelFile(path, file_name) => {
                    skel_path = Some((path, file_name));
                }
                AnimAction::ShapeIndex(_) => {}
                AnimAction::Frame(_, _) => {}
            }
        }

        info!("skel_path: {:?}", skel_path);

        Ok(FileReadOutput::Anim(skel_path))
    }

    pub fn post_process(
        commands: &mut Commands,
        server: &mut Server,
        project: &mut Project,
        file_key: &FileKey,
        file_entity: &Entity,
        skel_path_opt: Option<(String, String)>,
    ) -> HashMap<Entity, ContentEntityData> {
        let mut content_entities = HashMap::new();

        info!("skel_path: {:?}", skel_path_opt);
        if let Some((skel_path, skel_file_name)) = skel_path_opt {
            let skel_file_key = FileKey::new(&skel_path, &skel_file_name, EntryKind::File);
            let file_extension = project.file_extension(&skel_file_key).unwrap();
            if file_extension != FileExtension::Skel {
                panic!("anim file should depend on a single .skel file");
            }

            project.file_add_dependency(file_key, &skel_file_key);

            let skel_file_entity = project.file_entity(&skel_file_key).unwrap();

            // get all users in room with file entity
            let mut component = FileDependency::new();
            component.file_entity.set(server, file_entity);
            component.dependency_entity.set(server, &skel_file_entity);
            let entity = commands
                .spawn_empty()
                .enable_replication(server)
                .configure_replication(ReplicationConfig::Delegated)
                .insert(component)
                .id();
            content_entities.insert(entity, ContentEntityData::new_dependency(skel_file_key));
        }

        content_entities
    }
}

impl FileReader for AnimReader {
    fn read(
        &self,
        commands: &mut Commands,
        server: &mut Server,
        bytes: &Box<[u8]>,
    ) -> FileReadOutput {
        let mut bit_reader = BitReader::new(bytes);

        let Ok(actions) = Self::read_to_actions(&mut bit_reader) else {
            panic!("Error reading .anim file");
        };

        let Ok(result) = Self::actions_to_world(commands, server, actions) else {
            panic!("Error reading .anim file");
        };

        result
    }
}
