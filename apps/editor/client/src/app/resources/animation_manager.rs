use std::{
    collections::{HashMap, HashSet},
    f32::consts::FRAC_PI_2,
};

use bevy_ecs::{
    entity::Entity,
    prelude::{Commands, Query, With},
    system::{Res, ResMut, Resource, SystemState},
    world::{Mut, World},
};
use logging::{info, warn};

use naia_bevy_client::{Client, CommandsExt, Instant, ReplicationConfig};

use math::{
    convert_3d_to_2d, quat_from_spin_direction, spin_direction_from_quat, Mat4, Quat, Vec2, Vec3,
};
use render_api::{
    base::{Color, CpuMaterial, CpuMesh},
    components::{Camera, CameraProjection, Projection, RenderLayer, Transform, Visibility},
    resources::RenderFrame,
    shapes::{
        angle_between, get_2d_line_transform_endpoint, rotation_diff, set_2d_line_transform,
        set_2d_line_transform_from_angle, Circle, Line,
    },
};
use storage::{Handle, Storage};

use editor_proto::components::{
    AnimFrame, AnimRotation, EdgeAngle, FileExtension, ShapeName, Transition, Vertex3d, VertexRoot,
};

use crate::app::{
    components::{Edge2dLocal, LocalAnimRotation, Vertex2d},
    get_new_3d_position,
    plugin::Main,
    resources::{
        camera_manager::{set_camera_transform, CameraManager},
        canvas::Canvas,
        edge_manager::EdgeManager,
        input::CardinalDirection,
        tab_manager::TabManager,
        vertex_manager::VertexManager,
    },
};

struct FrameData {
    rotations: HashSet<Entity>,
}

impl FrameData {
    pub fn new() -> Self {
        Self {
            rotations: HashSet::new(),
        }
    }

    pub fn add_rotation(&mut self, rotation_entity: Entity) {
        self.rotations.insert(rotation_entity);
    }

    pub fn remove_rotation(&mut self, rotation_entity: &Entity) {
        self.rotations.remove(rotation_entity);
    }
}

struct FileFrameData {
    frames: HashMap<Entity, FrameData>,
    frame_list: Vec<Option<Entity>>,
}

impl FileFrameData {
    pub fn new() -> Self {
        Self {
            frames: HashMap::new(),
            frame_list: Vec::new(),
        }
    }

    pub fn register_frame(&mut self, frame_entity: Entity) {
        self.frames.insert(frame_entity, FrameData::new());
    }

    pub fn deregister_frame(&mut self, frame_entity: &Entity) {
        self.frames.remove(frame_entity);
    }

    pub fn add_rotation(&mut self, frame_entity: Entity, rotation_entity: Entity) {
        let frame_data = self.frames.get_mut(&frame_entity).unwrap();
        frame_data.add_rotation(rotation_entity);
    }

    pub fn remove_rotation(&mut self, frame_entity: &Entity, rotation_entity: &Entity) {
        let frame_data = self.frames.get_mut(&frame_entity).unwrap();
        frame_data.remove_rotation(rotation_entity);
    }

    pub fn count(&self) -> usize {
        let mut count = 0;
        for val_opt in &self.frame_list {
            if val_opt.is_some() {
                count += 1;
            }
        }
        count
    }
}

#[derive(Resource)]
pub struct AnimationManager {
    posing: bool,
    frame_size: Vec2,
    frame_buffer: Vec2,
    frame_hover: Option<usize>,
    resync_frame_order: HashSet<Entity>,
    last_rotation_dragged: Option<(Entity, Option<Quat>, Quat)>,

    pub current_skel_file: Option<Entity>,
    current_frame_index: usize,
    // file_entity -> file_frame_data
    frame_data: HashMap<Entity, FileFrameData>,
    // frame entity -> file_entity
    frames: HashMap<Entity, Entity>,
    // rotation_entity -> (frame_entity, vertex_name)
    rotations: HashMap<Entity, (Entity, String)>,
    // (frame_entity, vertex_name) -> rotation_entity
    vertex_names: HashMap<(Entity, String), Entity>,
    //
    framing_y: f32,

    preview_playing: bool,
    last_preview_instant: Instant,
    preview_elapsed_ms: f32,
    preview_frame_index: usize,
    preview_frame_selected: bool,

    framing_rotation: Vec2,
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self {
            posing: false,
            resync_frame_order: HashSet::new(),
            frame_size: Vec2::new(100.0, 100.0),
            frame_buffer: Vec2::new(12.0, 12.0),
            frame_hover: None,
            last_rotation_dragged: None,

            current_skel_file: None,
            current_frame_index: 0,
            frame_data: HashMap::new(),
            frames: HashMap::new(),
            rotations: HashMap::new(),
            vertex_names: HashMap::new(),
            framing_y: 0.0,

            preview_playing: false,
            last_preview_instant: Instant::now(),
            preview_elapsed_ms: 0.0,
            preview_frame_index: 0,
            preview_frame_selected: false,

            framing_rotation: Vec2::new(180.0, 0.0),
        }
    }
}

impl AnimationManager {
    pub(crate) fn current_frame_index(&self) -> usize {
        self.current_frame_index
    }

    pub fn set_current_frame_index(&mut self, frame_index: usize) {
        self.current_frame_index = frame_index;
    }

    pub fn preview_is_playing(&self) -> bool {
        self.preview_playing
    }

    pub fn preview_play(&mut self) {
        self.preview_playing = true;
        self.last_preview_instant = Instant::now();
    }

    pub fn preview_pause(&mut self) {
        self.preview_playing = false;
    }

    pub fn preview_frame_index(&self) -> usize {
        self.preview_frame_index
    }

    pub fn preview_elapsed_ms(&self) -> f32 {
        self.preview_elapsed_ms
    }

    pub fn get_rotations_frame_entity(&self, entity: &Entity) -> Option<Entity> {
        self.rotations
            .get(entity)
            .map(|(frame_entity, _)| *frame_entity)
    }

    pub fn get_frame_entity(&self, file_entity: &Entity, frame_index: usize) -> Option<Entity> {
        //info!("get_frame_entity({:?}, {:?})", file_entity, frame_index);
        let frame_data = self.frame_data.get(file_entity)?;
        //info!("frame list: {:?}", frame_data.frame_list);
        let entity_opt = frame_data.frame_list.get(frame_index)?.as_ref();
        let entity = entity_opt?;
        Some(*entity)
    }

    pub fn get_frame_rotations(
        &self,
        file_entity: &Entity,
        frame_entity: &Entity,
    ) -> Option<&HashSet<Entity>> {
        let frame_data = self.frame_data.get(file_entity)?;
        let frame_data = frame_data.frames.get(frame_entity)?;
        Some(&frame_data.rotations)
    }

    pub(crate) fn current_frame_entity(&self, file_entity: &Entity) -> Option<Entity> {
        let current_frame_index = self.current_frame_index;
        let frame_data = self.frame_data.get(file_entity)?; //&(*file_entity, current_frame_index)).copied()
        let entity_opt = frame_data.frame_list.get(current_frame_index)?.as_ref();
        let entity = entity_opt?;
        Some(*entity)
    }

    pub(crate) fn get_frame_count(&self, file_entity: &Entity) -> Option<usize> {
        let frame_data = self.frame_data.get(file_entity)?;
        Some(frame_data.frame_list.len())
    }

    pub(crate) fn register_frame(&mut self, file_entity: Entity, frame_entity: Entity) {
        if !self.frame_data.contains_key(&file_entity) {
            self.frame_data.insert(file_entity, FileFrameData::new());
        }
        let frame_data = self.frame_data.get_mut(&file_entity).unwrap();
        frame_data.register_frame(frame_entity);

        self.frames.insert(frame_entity, file_entity);

        self.framing_queue_resync_frame_order(&file_entity);
    }

    pub(crate) fn deregister_frame(&mut self, file_entity: &Entity, frame_entity: &Entity) {
        if !self.frame_data.contains_key(file_entity) {
            panic!("Frame data not found!");
        }

        let frame_data = self.frame_data.get_mut(file_entity).unwrap();
        frame_data.deregister_frame(frame_entity);

        if frame_data.frames.is_empty() {
            self.frame_data.remove(file_entity);
        }

        self.frames.remove(frame_entity);

        self.framing_queue_resync_frame_order(file_entity);

        // TODO: handle current selected frame ... harder to do because can we really suppose that
        // the current tab file entity is the same as the file entity here?
    }

    pub(crate) fn register_rotation(
        &mut self,
        frame_entity: Entity,
        rotation_entity: Entity,
        vertex_name: String,
    ) {
        self.rotations
            .insert(rotation_entity, (frame_entity, vertex_name.clone()));
        self.vertex_names
            .insert((frame_entity, vertex_name), rotation_entity);
        let file_entity = self.frames.get(&frame_entity).unwrap();
        let frame_data = self.frame_data.get_mut(file_entity).unwrap();
        frame_data.add_rotation(frame_entity, rotation_entity);
    }

    pub(crate) fn deregister_rotation(&mut self, rotation_entity: &Entity) {
        let (frame_entity, vertex_name) = self.rotations.remove(rotation_entity).unwrap();
        self.vertex_names.remove(&(frame_entity, vertex_name));

        if let Some(file_entity) = self.frames.get(&frame_entity) {
            if let Some(frame_data) = self.frame_data.get_mut(file_entity) {
                frame_data.remove_rotation(&frame_entity, rotation_entity);
            }
        }
    }

    pub fn is_posing(&self) -> bool {
        self.posing
    }

    pub fn is_framing(&self) -> bool {
        !self.posing
    }

    pub fn set_posing(&mut self, canvas: &mut Canvas) {
        self.posing = true;

        canvas.queue_resync_shapes();
    }

    pub fn set_framing(&mut self) {
        self.posing = false;
        self.preview_frame_selected = false;
    }

    pub fn set_preview_frame_selected(&mut self) {
        self.preview_frame_selected = true;
    }

    pub fn preview_frame_selected(&self) -> bool {
        self.preview_frame_selected
    }

    pub fn get_current_rotation(&self, file_entity: &Entity, vertex_name: &str) -> Option<&Entity> {
        let current_frame = self.current_frame_entity(file_entity)?;
        self.vertex_names
            .get(&(current_frame, vertex_name.to_string()))
    }

    pub fn reset_last_rotation_dragged(&mut self) {
        self.last_rotation_dragged = None;
    }

    pub fn take_last_rotation_dragged(&mut self) -> Option<(Entity, Option<Quat>, Quat)> {
        self.last_rotation_dragged.take()
    }

    pub fn create_networked_rotation(
        &mut self,
        commands: &mut Commands,
        client: &mut Client<Main>,
        frame_entity: Entity,
        name: String,
        rotation: Quat,
    ) -> Entity {
        let mut component = AnimRotation::new(name.clone(), rotation.into());
        component.frame_entity.set(client, &frame_entity);
        let new_rotation_entity = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication::<Main>(ReplicationConfig::Delegated)
            .insert(component)
            .id();

        self.rotation_postprocess(commands, frame_entity, new_rotation_entity, name);

        return new_rotation_entity;
    }

    pub fn rotation_postprocess(
        &mut self,
        commands: &mut Commands,
        frame_entity: Entity,
        rotation_entity: Entity,
        vertex_name: String,
    ) {
        self.register_rotation(frame_entity, rotation_entity, vertex_name);

        commands
            .entity(rotation_entity)
            .insert(LocalAnimRotation::new());
    }

    pub(crate) fn drag_vertex(
        &mut self,
        world: &mut World,
        file_entity: &Entity,
        vertex_3d_entity: Entity,
        vertex_2d_entity: Entity,
        mouse_position: Vec2,
    ) {
        if !self.posing {
            panic!("Not posing!");
        }

        // get rotation
        let Some(frame_entity) = self.current_frame_entity(file_entity) else {
            info!("no frame");
            return;
        };

        let Ok(shape_name) = world.query::<&ShapeName>().get(world, vertex_3d_entity) else {
            return;
        };

        let shape_name: String = (*shape_name.value).clone();

        let mut system_state: SystemState<(
            Commands,
            Client<Main>,
            ResMut<Canvas>,
            Res<CameraManager>,
            Res<VertexManager>,
            Res<EdgeManager>,
            Query<(&Camera, &Projection)>,
            Query<&Transform>,
            Query<&Vertex3d>,
            Query<(&mut AnimRotation, &LocalAnimRotation)>,
            Query<&ShapeName>,
            Query<&EdgeAngle>,
        )> = SystemState::new(world);
        let (
            mut commands,
            mut client,
            mut canvas,
            camera_manager,
            vertex_manager,
            edge_manager,
            camera_q,
            transform_q,
            vertex_3d_q,
            mut rotation_q,
            name_q,
            edge_angle_q,
        ) = system_state.get_mut(world);

        //
        let rotation_entity_opt = self.get_current_rotation(file_entity, &shape_name).copied();
        if let Some(rotation_entity) = rotation_entity_opt {
            if !Self::rotation_has_auth(&mut commands, &mut client, rotation_entity) {
                return;
            }
        }

        // get parent 3d position
        let parent_vertex_3d_entity = vertex_manager
            .vertex_parent_3d_entity(&vertex_3d_entity)
            .unwrap();
        let parent_original_3d_position =
            vertex_3d_q.get(parent_vertex_3d_entity).unwrap().as_vec3();
        let parent_rotated_3d_position = transform_q
            .get(parent_vertex_3d_entity)
            .unwrap()
            .translation;

        // get old 3d position
        let original_3d_position = vertex_3d_q.get(vertex_3d_entity).unwrap().as_vec3();

        // get edge entity
        let edge_3d_entity = edge_manager
            .edge_3d_entity_from_vertices(
                &vertex_manager,
                vertex_3d_entity,
                parent_vertex_3d_entity,
            )
            .unwrap();
        let edge_old_angle = edge_angle_q.get(edge_3d_entity).unwrap();
        let edge_old_angle: f32 = edge_old_angle.get_radians();

        let new_3d_position = get_new_3d_position(
            &camera_manager,
            &camera_q,
            &transform_q,
            &mouse_position,
            &vertex_2d_entity,
        );

        let base_direction = (original_3d_position - parent_original_3d_position).normalize();
        let target_direction = (new_3d_position - parent_rotated_3d_position).normalize();
        let mut rotation_angle =
            quat_from_spin_direction(edge_old_angle, base_direction, target_direction);

        get_inversed_final_rotation(
            &vertex_manager,
            self,
            frame_entity,
            parent_vertex_3d_entity,
            &name_q,
            &rotation_q,
            &mut rotation_angle,
        );

        self.update_or_create_rotation(
            vertex_2d_entity,
            frame_entity,
            shape_name,
            &mut commands,
            &mut client,
            &mut rotation_q,
            rotation_entity_opt,
            rotation_angle,
        );

        canvas.queue_resync_shapes();

        system_state.apply(world);
    }

    pub(crate) fn drag_edge(
        &mut self,
        world: &mut World,
        file_entity: &Entity,
        edge_3d_entity: Entity,
        edge_2d_entity: Entity,
        mouse_position: Vec2,
    ) {
        if !self.posing {
            panic!("Not posing!");
        }

        let Some(frame_entity) = self.current_frame_entity(file_entity) else {
            return;
        };

        let (_, vertex_3d_entity) = world
            .get_resource::<EdgeManager>()
            .unwrap()
            .edge_get_endpoints(&edge_3d_entity);
        let vertex_2d_entity = world
            .get_resource::<VertexManager>()
            .unwrap()
            .vertex_entity_3d_to_2d(&vertex_3d_entity)
            .unwrap();

        let Ok(shape_name) = world.query::<&ShapeName>().get(world, vertex_3d_entity) else {
            return;
        };

        let shape_name: String = (*shape_name.value).clone();

        let mut system_state: SystemState<(
            Commands,
            Client<Main>,
            ResMut<Canvas>,
            Res<VertexManager>,
            Res<EdgeManager>,
            Query<&Transform>,
            Query<&Vertex3d>,
            Query<&EdgeAngle>,
            Query<(&mut AnimRotation, &LocalAnimRotation)>,
            Query<&ShapeName>,
        )> = SystemState::new(world);
        let (
            mut commands,
            mut client,
            mut canvas,
            vertex_manager,
            edge_manager,
            transform_q,
            vertex_3d_q,
            edge_angle_q,
            mut rotation_q,
            name_q,
        ) = system_state.get_mut(world);

        //
        let rotation_entity_opt = self
            .get_current_rotation(&file_entity, &shape_name)
            .copied();
        if let Some(rotation_entity) = rotation_entity_opt {
            if !Self::rotation_has_auth(&mut commands, &mut client, rotation_entity) {
                return;
            }
        }

        //
        let edge_2d_transform = transform_q.get(edge_2d_entity).unwrap();
        let edge_start_pos = edge_2d_transform.translation.truncate();
        let edge_end_pos = get_2d_line_transform_endpoint(&edge_2d_transform);
        let edge_base_angle = angle_between(&edge_start_pos, &edge_end_pos);

        let edge_angle_entity = edge_manager.edge_get_base_circle_entity(&edge_3d_entity);
        let edge_angle_pos = transform_q
            .get(edge_angle_entity)
            .unwrap()
            .translation
            .truncate();

        let edge_old_angle = edge_angle_q.get(edge_3d_entity).unwrap();
        let edge_old_angle: f32 = edge_old_angle.get_radians();
        let edge_new_angle =
            angle_between(&edge_angle_pos, &mouse_position) - FRAC_PI_2 - edge_base_angle;
        let edge_diff_angle = rotation_diff(edge_old_angle, edge_new_angle);
        //

        // get parent 3d position
        let parent_vertex_3d_entity = vertex_manager
            .vertex_parent_3d_entity(&vertex_3d_entity)
            .unwrap();
        let parent_original_3d_position =
            vertex_3d_q.get(parent_vertex_3d_entity).unwrap().as_vec3();
        let parent_rotated_3d_position = transform_q
            .get(parent_vertex_3d_entity)
            .unwrap()
            .translation;

        // get old 3d position
        let original_3d_position = vertex_3d_q.get(vertex_3d_entity).unwrap().as_vec3();
        let rotated_3d_position = transform_q.get(vertex_3d_entity).unwrap().translation;

        let base_direction = (original_3d_position - parent_original_3d_position).normalize();
        let target_direction = (rotated_3d_position - parent_rotated_3d_position).normalize();
        let mut rotation_angle =
            quat_from_spin_direction(edge_diff_angle, base_direction, target_direction);

        get_inversed_final_rotation(
            &vertex_manager,
            self,
            frame_entity,
            parent_vertex_3d_entity,
            &name_q,
            &rotation_q,
            &mut rotation_angle,
        );

        self.update_or_create_rotation(
            vertex_2d_entity,
            frame_entity,
            shape_name,
            &mut commands,
            &mut client,
            &mut rotation_q,
            rotation_entity_opt,
            rotation_angle,
        );

        canvas.queue_resync_shapes();

        system_state.apply(world);
    }

    pub(crate) fn handle_mouse_drag_anim_framing(&mut self, delta_y: f32) {
        self.framing_y += delta_y;
    }

    pub(crate) fn sync_shapes_3d(
        &self,
        world: &mut World,
        vertex_manager: &VertexManager,
        camera_3d_scale: f32,
        frame_entity: Entity,
        // option<(frame_entity, interp amount)>
        interp_opt: Option<(Entity, f32)>,
        root_3d_vertex: Entity,
    ) {
        let mut system_state: SystemState<(
            Res<EdgeManager>,
            Query<(Entity, &Vertex3d)>,
            Query<&mut Transform>,
            Query<&mut Visibility>,
            Query<&ShapeName>,
            Query<(&AnimRotation, &mut LocalAnimRotation)>,
            Query<&EdgeAngle>,
        )> = SystemState::new(world);
        let (
            edge_manager,
            vertex_3d_q,
            mut transform_q,
            mut visibility_q,
            name_q,
            mut rotation_q,
            edge_angle_q,
        ) = system_state.get_mut(world);

        let (_, vertex_3d) = vertex_3d_q.get(root_3d_vertex).unwrap();
        let vertex_pos = vertex_3d.as_vec3();

        self.sync_shapes_3d_children(
            vertex_manager,
            &edge_manager,
            &vertex_3d_q,
            &mut transform_q,
            &mut visibility_q,
            &name_q,
            &mut rotation_q,
            &edge_angle_q,
            camera_3d_scale,
            frame_entity,
            interp_opt,
            root_3d_vertex,
            vertex_pos,
            vertex_pos,
            Quat::IDENTITY,
        );
    }

    fn sync_shapes_3d_children(
        &self,
        vertex_manager: &VertexManager,
        edge_manager: &EdgeManager,
        vertex_3d_q: &Query<(Entity, &Vertex3d)>,
        transform_q: &mut Query<&mut Transform>,
        visibility_q: &mut Query<&mut Visibility>,
        name_q: &Query<&ShapeName>,
        rotation_q: &mut Query<(&AnimRotation, &mut LocalAnimRotation)>,
        edge_angle_q: &Query<&EdgeAngle>,
        camera_3d_scale: f32,
        frame_entity: Entity,
        // option<(frame_entity, interp amount)>
        interp_opt: Option<(Entity, f32)>,
        parent_vertex_3d_entity: Entity,
        original_parent_pos: Vec3,
        rotated_parent_pos: Vec3,
        parent_rotation: Quat,
    ) {
        // sync children vertices
        let Some(children) = vertex_manager.vertex_children_3d_entities(&parent_vertex_3d_entity)
        else {
            return;
        };

        for child_vertex_3d_entity in children.iter() {
            let (_, vertex_3d) = vertex_3d_q.get(*child_vertex_3d_entity).unwrap();
            let original_child_pos = vertex_3d.as_vec3();

            // a lot of this should be refactored to share code with edge_manager.rs
            let mut child_rotation = Quat::IDENTITY;
            if let Ok(name_component) = name_q.get(*child_vertex_3d_entity) {
                let name = (*name_component.value).clone();
                if let Some(rotation_entity) = self.vertex_names.get(&(frame_entity, name.clone()))
                {
                    if let Ok((anim_rotation, mut local_anim_rotation)) =
                        rotation_q.get_mut(*rotation_entity)
                    {
                        child_rotation = anim_rotation.get_rotation();
                        local_anim_rotation.last_synced_quat = child_rotation;
                    }
                }
                if let Some((interp_frame_entity, interp_amount)) = interp_opt {
                    let interp_rotation = if let Some(rotation_entity) =
                        self.vertex_names.get(&(interp_frame_entity, name))
                    {
                        if let Ok((anim_rotation, _)) = rotation_q.get(*rotation_entity) {
                            anim_rotation.get_rotation()
                        } else {
                            Quat::IDENTITY
                        }
                    } else {
                        Quat::IDENTITY
                    };

                    child_rotation = child_rotation.slerp(interp_rotation, interp_amount);
                }
            }

            let child_rotation = (parent_rotation * child_rotation).normalize();
            let original_child_displacement = original_child_pos - original_parent_pos;
            let rotated_child_displacement = child_rotation * original_child_displacement;
            let rotated_child_pos = rotated_parent_pos + rotated_child_displacement;

            // update vertex transform
            let Ok(mut vertex_3d_transform) = transform_q.get_mut(*child_vertex_3d_entity) else {
                warn!(
                    "Vertex3d entity {:?} has no Transform",
                    child_vertex_3d_entity
                );
                continue;
            };
            vertex_3d_transform.translation = rotated_child_pos;

            // update edge transform
            if let Some(edge_3d_entity) = edge_manager.edge_3d_entity_from_vertices(
                vertex_manager,
                parent_vertex_3d_entity,
                *child_vertex_3d_entity,
            ) {
                let Ok(mut edge_3d_transform) = transform_q.get_mut(edge_3d_entity) else {
                    warn!("Edge3d entity {:?} has no Transform", edge_3d_entity);
                    continue;
                };

                // get edge angle
                let edge_spin = match edge_angle_q.get(edge_3d_entity) {
                    Ok(edge_angle) => edge_angle.get_radians(),
                    Err(_) => 0.0,
                };

                let (edge_quat, scale) = get_3d_line_rotation_and_scale(
                    original_parent_pos,
                    original_child_pos,
                    edge_spin,
                );

                edge_3d_transform.translation = rotated_parent_pos;
                edge_3d_transform.rotation = child_rotation * edge_quat;
                edge_3d_transform.scale.x = scale;

                // update edge angle 2d representation
                sync_edge_angle(
                    edge_manager,
                    transform_q,
                    visibility_q,
                    camera_3d_scale,
                    edge_3d_entity,
                    child_rotation,
                    original_child_displacement.normalize(),
                    edge_spin,
                );
            }

            // recurse
            self.sync_shapes_3d_children(
                vertex_manager,
                edge_manager,
                vertex_3d_q,
                transform_q,
                visibility_q,
                name_q,
                rotation_q,
                edge_angle_q,
                camera_3d_scale,
                frame_entity,
                interp_opt,
                *child_vertex_3d_entity,
                original_child_pos,
                rotated_child_pos,
                child_rotation,
            );
        }
    }

    fn update_or_create_rotation(
        &mut self,
        vertex_2d_entity: Entity,
        frame_entity: Entity,
        shape_name: String,
        mut commands: &mut Commands,
        mut client: &mut Client<Main>,
        rotation_q: &mut Query<(&mut AnimRotation, &LocalAnimRotation)>,
        rotation_entity_opt: Option<Entity>,
        rotation_angle: Quat,
    ) {
        if let Some(rotation_entity) = rotation_entity_opt {
            let (mut anim_rotation, _) = rotation_q.get_mut(rotation_entity).unwrap();

            self.update_last_rotation_dragged(
                vertex_2d_entity,
                Some(anim_rotation.get_rotation()),
                rotation_angle,
            );

            anim_rotation.set_rotation(rotation_angle);
        } else {
            self.update_last_rotation_dragged(vertex_2d_entity, None, rotation_angle);

            // create new rotation entity
            self.create_networked_rotation(
                &mut commands,
                &mut client,
                frame_entity,
                shape_name.to_string(),
                rotation_angle,
            );
        };
    }

    fn rotation_has_auth(
        commands: &mut Commands,
        client: &mut Client<Main>,
        rotation_entity: Entity,
    ) -> bool {
        let auth_status = commands.entity(rotation_entity).authority(&client).unwrap();
        if !(auth_status.is_requested() || auth_status.is_granted()) {
            // only continue to mutate if requested or granted authority over vertex
            info!("No authority over vertex rotation, skipping..");
            return false;
        }
        return true;
    }

    fn update_last_rotation_dragged(
        &mut self,
        vertex_2d_entity: Entity,
        old_rotation: Option<Quat>,
        new_rotation: Quat,
    ) {
        if let Some((_, old_rotation, _)) = self.last_rotation_dragged {
            self.last_rotation_dragged = Some((vertex_2d_entity, old_rotation, new_rotation));
        } else {
            self.last_rotation_dragged = Some((vertex_2d_entity, old_rotation, new_rotation));
        }
    }

    // Framing

    pub fn frame_index_hover(&self) -> Option<usize> {
        self.frame_hover
    }

    pub fn framing_navigate(
        &mut self,
        current_file_entity: &Entity,
        dir: CardinalDirection,
    ) -> Option<(usize, usize)> {
        let mut current_index = self.current_frame_index;
        let Some(frame_data) = self.frame_data.get(current_file_entity) else {
            return None;
        };
        match dir {
            CardinalDirection::West => {
                if current_index <= 0 {
                    return None;
                }
                current_index -= 1;
                // if no frame entity, continue decrementing
                while frame_data.frame_list[current_index].is_none() {
                    if current_index <= 0 {
                        return None;
                    }
                    current_index -= 1;
                }
                return Some((self.current_frame_index, current_index));
            }
            CardinalDirection::East => {
                if current_index >= frame_data.frame_list.len() - 1 {
                    return None;
                }
                current_index += 1;
                // if no frame entity, continue incrementing
                while frame_data.frame_list[current_index].is_none() {
                    if current_index >= frame_data.frame_list.len() - 1 {
                        return None;
                    }
                    current_index += 1;
                }
                return Some((self.current_frame_index, current_index));
            }
            _ => {
                return None;
            }
        }
    }

    pub fn framing_handle_mouse_wheel(&mut self, scroll_y: f32) {
        let scroll_y = 0.8 + (((scroll_y + 24.0) / 48.0) * 0.4);
        self.frame_size *= scroll_y;
        if self.frame_size.x < 40.0 {
            self.frame_size = Vec2::new(40.0, 40.0);
        }
        if self.frame_size.x > 400.0 {
            self.frame_size = Vec2::new(400.0, 400.0);
        }
    }

    pub fn sync_mouse_hover_ui_framing(
        &mut self,
        current_file_entity: &Entity,
        canvas_size: Vec2,
        mouse_position: &Vec2,
    ) {
        let Some(file_frame_data) = self.frame_data.get(current_file_entity) else {
            return;
        };

        let frame_count = file_frame_data.count();

        let frame_positions = self.get_frame_positions(canvas_size, frame_count);

        self.frame_hover = None;
        for (index, frame_position) in frame_positions.iter().enumerate() {
            // assign hover frame
            if mouse_position.x >= frame_position.x
                && mouse_position.x <= frame_position.x + self.frame_size.x
            {
                if mouse_position.y >= frame_position.y
                    && mouse_position.y <= frame_position.y + self.frame_size.y
                {
                    self.frame_hover = Some(index);
                    return;
                }
            }
        }
    }

    pub fn draw_framing(&mut self, world: &mut World) {
        // get current file
        let Some(current_file_entity) = world
            .get_resource::<TabManager>()
            .unwrap()
            .current_tab_entity()
        else {
            return;
        };
        let current_file_entity = *current_file_entity;

        let Some(file_frame_data) = self.frame_data.get(&current_file_entity) else {
            return;
        };

        let frame_count = file_frame_data.count();
        let canvas_size = world.get_resource::<Canvas>().unwrap().texture_size();
        let frame_rects = self.get_frame_positions(canvas_size, frame_count);

        let file_frame_data = self.frame_data.get(&current_file_entity).unwrap();

        let (
            frame_rects,
            render_layer,
            camera_3d_entity,
            point_mesh_handle,
            line_mesh_handle,
            mat_handle_white,
            mat_handle_green,
        ) = {
            // draw
            let mut system_state: SystemState<(
                ResMut<RenderFrame>,
                Res<CameraManager>,
                ResMut<Storage<CpuMesh>>,
                ResMut<Storage<CpuMaterial>>,
                Query<(&mut Camera, &mut Projection, &mut Transform)>,
            )> = SystemState::new(world);
            let (mut render_frame, camera_manager, mut meshes, mut materials, mut camera_q) =
                system_state.get_mut(world);

            camera_manager.enable_cameras(&mut camera_q, true);

            let render_layer = camera_manager.layer_2d;
            let camera_3d_entity = camera_manager.camera_3d_entity().unwrap();
            let point_mesh_handle = meshes.add(Circle::new(Vertex2d::SUBDIVISIONS));
            let line_mesh_handle = meshes.add(Line);
            let mat_handle_white = materials.add(Color::WHITE);
            let mat_handle_gray = materials.add(Color::GRAY);
            let mat_handle_dark_gray = materials.add(Color::DARK_GRAY);
            let mat_handle_green = materials.add(Color::GREEN);

            for (frame_index, frame_pos) in frame_rects.iter().enumerate() {
                // frame_index 0 is preview frame
                let selected: bool = frame_index > 0 && self.current_frame_index == frame_index - 1;

                // set thickness to 4.0 if frame is hovered and not currently selected, otherwise 2.0
                let thickness = if !selected && Some(frame_index) == self.frame_hover {
                    4.0
                } else {
                    2.0
                };

                let mat = if frame_index == 0 {
                    mat_handle_dark_gray
                } else {
                    mat_handle_gray
                };

                draw_rectangle(
                    &mut render_frame,
                    &render_layer,
                    &line_mesh_handle,
                    &mat,
                    *frame_pos,
                    self.frame_size,
                    thickness,
                );

                if selected {
                    // draw white rectangle around selected frame
                    draw_rectangle(
                        &mut render_frame,
                        &render_layer,
                        &line_mesh_handle,
                        &mat_handle_white,
                        *frame_pos + Vec2::new(-4.0, -4.0),
                        self.frame_size + Vec2::new(8.0, 8.0),
                        thickness,
                    );
                }
            }

            (
                frame_rects,
                render_layer,
                camera_3d_entity,
                point_mesh_handle,
                line_mesh_handle,
                mat_handle_white,
                mat_handle_green,
            )
        };

        let Some(root_3d_vertex) = get_root_vertex(world) else {
            return;
        };

        let mut system_state: SystemState<Query<(&Camera, &Projection)>> = SystemState::new(world);
        let camera_q = system_state.get_mut(world);
        let Ok((camera, camera_projection)) = camera_q.get(camera_3d_entity) else {
            return;
        };

        let camera_viewport = camera.viewport.unwrap();
        let projection_matrix = camera_projection.projection_matrix(&camera_viewport);

        let mut camera_transform = Transform::default();
        set_camera_transform(
            &mut camera_transform,
            self.framing_rotation,
            4.0,
            Vec2::ZERO,
        );
        let view_matrix = camera_transform.view_matrix();

        world.resource_scope(|world, vertex_manager: Mut<VertexManager>| {
            let mut frame_index = 0;

            {
                // draw preview frame
                if let Some(Some(preview_current_frame_entity)) =
                    file_frame_data.frame_list.get(self.preview_frame_index)
                {
                    let mut preview_next_frame_index = self.preview_frame_index + 1;
                    if preview_next_frame_index >= frame_count {
                        preview_next_frame_index -= frame_count;
                    }
                    if let Some(preview_next_frame_entity) =
                        file_frame_data.frame_list[preview_next_frame_index]
                    {
                        let frame_pos = frame_rects[frame_index];
                        if let Ok(frame_component) = world
                            .query::<&AnimFrame>()
                            .get(world, *preview_current_frame_entity)
                        {
                            let frame_duration =
                                frame_component.transition.get_duration_ms() as f32;
                            let interp_amount = self.preview_elapsed_ms / frame_duration;

                            self.draw_pose(
                                world,
                                &vertex_manager,
                                *preview_current_frame_entity,
                                Some((preview_next_frame_entity, interp_amount)),
                                root_3d_vertex,
                                &frame_pos,
                                &render_layer,
                                &point_mesh_handle,
                                &line_mesh_handle,
                                &mat_handle_green,
                                &view_matrix,
                                &projection_matrix,
                            );
                        }
                    }
                }

                frame_index += 1;
            }

            for frame_opt in file_frame_data.frame_list.iter() {
                if frame_opt.is_none() {
                    continue;
                }
                let frame_entity = frame_opt.unwrap();

                let frame_pos = frame_rects[frame_index];

                self.draw_pose(
                    world,
                    &vertex_manager,
                    frame_entity,
                    None,
                    root_3d_vertex,
                    &frame_pos,
                    &render_layer,
                    &point_mesh_handle,
                    &line_mesh_handle,
                    &mat_handle_green,
                    &view_matrix,
                    &projection_matrix,
                );

                frame_index += 1;
            }
        });

        self.draw_preview_time_line(
            world,
            &current_file_entity,
            &render_layer,
            &line_mesh_handle,
            &mat_handle_white,
            &frame_rects,
        );
    }

    fn draw_preview_time_line(
        &self,
        world: &mut World,
        current_file_entity: &Entity,
        render_layer: &RenderLayer,
        line_mesh_handle: &Handle<CpuMesh>,
        mat_handle_white: &Handle<CpuMaterial>,
        frame_positions: &Vec<Vec2>,
    ) {
        let Some(frame_entity) =
            self.get_frame_entity(current_file_entity, self.preview_frame_index)
        else {
            return;
        };
        let Ok(frame_component) = world.query::<&AnimFrame>().get(world, frame_entity) else {
            return;
        };
        let frame_duration = frame_component.transition.get_duration_ms() as f32;
        let complete = self.preview_elapsed_ms / frame_duration;
        let frame_width = self.frame_size.x + self.frame_buffer.x;
        let frame_count = frame_positions.len();

        let mut start: Vec2;
        if complete < 0.5 {
            let mut preview_frame_index = self.preview_frame_index + 1;
            if preview_frame_index >= frame_count {
                preview_frame_index -= frame_count - 1;
            }

            start = frame_positions[preview_frame_index];

            start.x += frame_width * complete;
        } else {
            let mut next_frame_index = self.preview_frame_index + 2;
            if next_frame_index >= frame_count {
                next_frame_index -= frame_count - 1;
            }
            start = frame_positions[next_frame_index];
            start.x -= frame_width * (1.0 - complete);
        }

        start.x += self.frame_size.x * 0.5;
        start.y -= self.frame_buffer.y;

        let mut end = start;
        end.y += self.frame_size.y + (self.frame_buffer.y * 2.0);

        let mut render_frame = world.get_resource_mut::<RenderFrame>().unwrap();
        draw_line(
            &mut render_frame,
            render_layer,
            line_mesh_handle,
            mat_handle_white,
            start,
            end,
            2.0,
        );
    }

    fn draw_pose(
        &self,
        world: &mut World,
        vertex_manager: &VertexManager,
        frame_entity: Entity,
        interp_opt: Option<(Entity, f32)>,
        root_3d_vertex: Entity,
        frame_pos: &Vec2,
        render_layer: &RenderLayer,
        point_mesh_handle: &Handle<CpuMesh>,
        line_mesh_handle: &Handle<CpuMesh>,
        mat_handle_green: &Handle<CpuMaterial>,
        view_matrix: &Mat4,
        projection_matrix: &Mat4,
    ) {
        self.sync_shapes_3d(
            world,
            vertex_manager,
            1.0,
            frame_entity,
            interp_opt,
            root_3d_vertex,
        );
        let mut frame_size = self.frame_size;
        frame_size.x *= 0.5;
        let root_pos = *frame_pos + frame_size;
        self.draw_shapes_3d(
            world,
            vertex_manager,
            root_3d_vertex,
            &root_pos,
            frame_pos,
            render_layer,
            point_mesh_handle,
            line_mesh_handle,
            mat_handle_green,
            view_matrix,
            projection_matrix,
        );
    }

    fn draw_shapes_3d(
        &self,
        world: &mut World,
        vertex_manager: &VertexManager,
        root_3d_vertex: Entity,
        root_pos: &Vec2,
        frame_pos: &Vec2,
        render_layer: &RenderLayer,
        point_mesh_handle: &Handle<CpuMesh>,
        line_mesh_handle: &Handle<CpuMesh>,
        mat_handle_green: &Handle<CpuMaterial>,
        view_matrix: &Mat4,
        projection_matrix: &Mat4,
    ) {
        let mut system_state: SystemState<(
            Res<EdgeManager>,
            ResMut<RenderFrame>,
            Query<&Transform>,
        )> = SystemState::new(world);
        let (edge_manager, mut render_frame, transform_q) = system_state.get_mut(world);

        self.draw_shapes_3d_children(
            vertex_manager,
            &edge_manager,
            &mut render_frame,
            &transform_q,
            root_3d_vertex,
            root_pos,
            render_layer,
            point_mesh_handle,
            line_mesh_handle,
            mat_handle_green,
            view_matrix,
            projection_matrix,
            frame_pos,
        );
    }

    fn draw_shapes_3d_children(
        &self,
        vertex_manager: &VertexManager,
        edge_manager: &EdgeManager,
        render_frame: &mut RenderFrame,
        transform_q: &Query<&Transform>,
        parent_vertex_3d_entity: Entity,
        parent_pos: &Vec2,
        render_layer: &RenderLayer,
        point_mesh_handle: &Handle<CpuMesh>,
        line_mesh_handle: &Handle<CpuMesh>,
        mat_handle_green: &Handle<CpuMaterial>,
        view_matrix: &Mat4,
        projection_matrix: &Mat4,
        frame_pos: &Vec2,
    ) {
        // sync children vertices
        let Some(children) = vertex_manager.vertex_children_3d_entities(&parent_vertex_3d_entity)
        else {
            return;
        };

        for child_vertex_3d_entity in children.iter() {
            // draw vertex 2d

            // get 3d transform
            let Ok(vertex_3d_transform) = transform_q.get(*child_vertex_3d_entity) else {
                warn!(
                    "Vertex3d entity {:?} has no Transform",
                    child_vertex_3d_entity
                );
                continue;
            };

            // update 2d vertices
            let (coords, _depth) = convert_3d_to_2d(
                view_matrix,
                projection_matrix,
                &self.frame_size,
                &vertex_3d_transform.translation,
            );

            let adjust = Vec2::new(0.0, self.frame_size.y * 0.5);
            let child_position = coords + *frame_pos + adjust;
            let child_transform = Transform::from_translation_2d(child_position);
            render_frame.draw_mesh(
                Some(render_layer),
                point_mesh_handle,
                &mat_handle_green,
                &child_transform,
            );

            // draw edge 2d
            let mut line_transform = Transform::default();
            set_2d_line_transform(&mut line_transform, *parent_pos, child_position, 0.0);
            render_frame.draw_mesh(
                Some(render_layer),
                line_mesh_handle,
                &mat_handle_green,
                &line_transform,
            );

            // recurse
            self.draw_shapes_3d_children(
                vertex_manager,
                edge_manager,
                render_frame,
                transform_q,
                *child_vertex_3d_entity,
                &child_position,
                render_layer,
                point_mesh_handle,
                line_mesh_handle,
                mat_handle_green,
                view_matrix,
                projection_matrix,
                frame_pos,
            );
        }
    }

    fn get_frame_positions(&mut self, canvas_size: Vec2, frame_count: usize) -> Vec<Vec2> {
        let mut positions = Vec::new();
        let mut start_position = self.frame_buffer;

        for _ in 0..=frame_count {
            positions.push(start_position);
            let next_x = start_position.x + self.frame_size.x + self.frame_buffer.x;
            if next_x + self.frame_size.x > canvas_size.x {
                start_position.x = self.frame_buffer.x;
                start_position.y += self.frame_size.y + self.frame_buffer.y;
            } else {
                start_position.x = next_x;
            }
        }

        let last_y = start_position.y + self.frame_size.y + self.frame_buffer.y;
        let y_diff = last_y - canvas_size.y;
        if y_diff <= 0.0 {
            self.framing_y = 0.0;
        } else {
            if self.framing_y > 0.0 {
                self.framing_y = 0.0;
            }
            if self.framing_y < -y_diff {
                self.framing_y = -y_diff;
            }
        }

        for position in positions.iter_mut() {
            position.y += self.framing_y;
        }

        positions
    }

    pub fn framing_queue_resync_frame_order(&mut self, file_entity: &Entity) {
        info!(
            "framing_queue_resync_frame_order for entity: `{:?}`",
            file_entity
        );
        self.resync_frame_order.insert(*file_entity);
    }

    pub fn framing_resync_frame_order(
        &mut self,
        client: &Client<Main>,
        frame_q: &Query<(Entity, &AnimFrame)>,
    ) {
        if self.resync_frame_order.is_empty() {
            return;
        }
        let resync_frame_order = std::mem::take(&mut self.resync_frame_order);
        for file_entity in resync_frame_order {
            // info!("resync_frame_order for entity: `{:?}`", file_entity);
            self.framing_recalc_order(client, &file_entity, frame_q);
        }
    }

    pub fn framing_insert_frame(
        &mut self,
        commands: &mut Commands,
        client: &mut Client<Main>,
        file_entity: Entity,
        frame_index: usize,
    ) -> Entity {
        let mut frame_component = AnimFrame::new(frame_index as u8, Transition::new(50));
        frame_component.file_entity.set(client, &file_entity);
        let entity_id = commands
            .spawn_empty()
            .enable_replication(client)
            .configure_replication::<Main>(ReplicationConfig::Delegated)
            .insert(frame_component)
            .id();

        // create new 2d vertex, add local components to 3d vertex
        self.frame_postprocess(file_entity, entity_id);

        entity_id
    }

    fn framing_recalc_order(
        &mut self,
        client: &Client<Main>,
        file_entity: &Entity,
        frame_q: &Query<(Entity, &AnimFrame)>,
    ) {
        let Some(frame_data) = self.frame_data.get_mut(&file_entity) else {
            return;
        };

        let mut new_frame_list = Vec::new();

        for (frame_entity, frame) in frame_q.iter() {
            let frames_file_entity = frame.file_entity.get(client).unwrap();
            if frames_file_entity != *file_entity {
                continue;
            }
            let frame_index = frame.get_order() as usize;
            // resize if necessary
            if frame_index >= new_frame_list.len() {
                new_frame_list.resize(frame_index + 1, None);
            }
            if new_frame_list[frame_index].is_some() {
                warn!("Duplicate frame order! {:?}", frame_index);
            }
            new_frame_list[frame_index] = Some(frame_entity);
        }

        for (index, frame_entity_opt) in new_frame_list.iter().enumerate() {
            info!("frame order: {:?} -> {:?}", index, frame_entity_opt);
        }

        frame_data.frame_list = new_frame_list;
    }

    pub(crate) fn frame_postprocess(&mut self, file_entity: Entity, frame_entity: Entity) {
        self.register_frame(file_entity, frame_entity);
    }

    pub(crate) fn preview_update(
        &mut self,
        canvas: &mut Canvas,
        current_file_entity: &Entity,
        frame_q: &Query<(Entity, &AnimFrame)>,
    ) {
        if !self.preview_playing {
            return;
        }

        let now = Instant::now();
        let ms_elapsed = self.last_preview_instant.elapsed(&now).as_millis() as f32;
        self.last_preview_instant = now;

        let Some(preview_frame_count) = self.get_frame_count(current_file_entity) else {
            return;
        };

        let Some(frame_entity) =
            self.get_frame_entity(current_file_entity, self.preview_frame_index)
        else {
            return;
        };
        let Ok((_, frame_component)) = frame_q.get(frame_entity) else {
            return;
        };
        let mut frame_duration = frame_component.transition.get_duration_ms() as f32;

        self.preview_elapsed_ms += ms_elapsed / 10.0; // change this back to 1 for real speeds! maybe should be configurable..
        while self.preview_elapsed_ms > frame_duration {
            self.preview_elapsed_ms -= frame_duration;
            self.preview_frame_index += 1;
            if self.preview_frame_index >= preview_frame_count {
                self.preview_frame_index = 0;
            }
            let Ok((_, frame_component)) = frame_q.get(frame_entity) else {
                break;
            };
            frame_duration = frame_component.transition.get_duration_ms() as f32;
        }

        canvas.queue_resync_shapes();
    }
}

fn draw_rectangle(
    render_frame: &mut RenderFrame,
    render_layer: &RenderLayer,
    mesh_handle: &Handle<CpuMesh>,
    mat_handle: &Handle<CpuMaterial>,
    position: Vec2,
    size: Vec2,
    thickness: f32,
) {
    // top
    let start = position;
    let mut end = position;
    end.x += size.x;
    draw_line(
        render_frame,
        render_layer,
        mesh_handle,
        mat_handle,
        start,
        end,
        thickness,
    );

    // bottom
    let mut start = position;
    start.y += size.y;
    let mut end = start;
    end.x += size.x;
    draw_line(
        render_frame,
        render_layer,
        mesh_handle,
        mat_handle,
        start,
        end,
        thickness,
    );

    // left
    let start = position;
    let mut end = position;
    end.y += size.y;
    draw_line(
        render_frame,
        render_layer,
        mesh_handle,
        mat_handle,
        start,
        end,
        thickness,
    );

    // right
    let mut start = position;
    start.x += size.x;
    let mut end = start;
    end.y += size.y;
    draw_line(
        render_frame,
        render_layer,
        mesh_handle,
        mat_handle,
        start,
        end,
        thickness,
    );
}

fn draw_line(
    render_frame: &mut RenderFrame,
    render_layer: &RenderLayer,
    mesh_handle: &Handle<CpuMesh>,
    mat_handle: &Handle<CpuMaterial>,
    start: Vec2,
    end: Vec2,
    thickness: f32,
) {
    let mut transform = Transform::default();
    transform.scale.y = thickness;
    set_2d_line_transform(&mut transform, start, end, 0.0);
    render_frame.draw_mesh(Some(render_layer), mesh_handle, mat_handle, &transform);
}

fn sync_edge_angle(
    edge_manager: &EdgeManager,
    transform_q: &mut Query<&mut Transform>,
    visibility_q: &mut Query<&mut Visibility>,
    camera_3d_scale: f32,
    edge_3d_entity: Entity,
    rotation: Quat,
    displacement: Vec3,
    edge_angle: f32,
) {
    // a lot of this should be refactored to share code with edge_manager.rs

    let edge_angles_visible = edge_manager.edge_angles_are_visible(FileExtension::Anim);

    let Some(edge_2d_entity) = edge_manager.edge_entity_3d_to_2d(&edge_3d_entity) else {
        return;
    };

    // visibility
    let Ok(visibility) = visibility_q.get(edge_2d_entity) else {
        panic!("entity has no Visibility");
    };
    if !visibility.visible {
        return;
    }

    let (base_circle_entity, angle_edge_entity, end_circle_entity) =
        edge_manager.edge_angle_entities(&edge_3d_entity).unwrap();

    for entity in [base_circle_entity, angle_edge_entity, end_circle_entity] {
        let Ok(mut visibility) = visibility_q.get_mut(entity) else {
            warn!("Edge angle entity {:?} has no transform", entity);
            continue;
        };
        visibility.visible = edge_angles_visible;
    }

    if edge_angles_visible {
        let edge_angle_base_circle_scale =
            Edge2dLocal::EDGE_ANGLE_BASE_CIRCLE_RADIUS * camera_3d_scale;
        let edge_angle_end_circle_scale =
            Edge2dLocal::EDGE_ANGLE_END_CIRCLE_RADIUS * camera_3d_scale;
        let edge_angle_length = Edge2dLocal::EDGE_ANGLE_LENGTH * camera_3d_scale;
        let edge_angle_thickness = Edge2dLocal::EDGE_ANGLE_THICKNESS * camera_3d_scale;

        let edge_2d_transform = transform_q.get(edge_2d_entity).unwrap();
        let start_pos = edge_2d_transform.translation.truncate();
        let end_pos = get_2d_line_transform_endpoint(&edge_2d_transform);
        let base_angle = angle_between(&start_pos, &end_pos);
        let middle_pos = (start_pos + end_pos) / 2.0;
        let edge_depth = edge_2d_transform.translation.z;

        let Ok(mut angle_transform) = transform_q.get_mut(angle_edge_entity) else {
            warn!("Edge angle entity {:?} has no transform", angle_edge_entity);
            return;
        };

        let (rotation_spin, _) = spin_direction_from_quat(displacement, rotation);

        let edge_angle_drawn = base_angle + edge_angle + FRAC_PI_2 - rotation_spin;
        let edge_depth_drawn = edge_depth - 1.0;
        set_2d_line_transform_from_angle(
            &mut angle_transform,
            middle_pos,
            edge_angle_drawn,
            edge_angle_length,
            edge_depth_drawn,
        );
        angle_transform.scale.y = edge_angle_thickness;
        let edge_angle_endpoint = get_2d_line_transform_endpoint(&angle_transform);

        let Ok(mut base_circle_transform) = transform_q.get_mut(base_circle_entity) else {
            warn!(
                "Edge angle base circle entity {:?} has no transform",
                base_circle_entity
            );
            return;
        };
        base_circle_transform.translation.x = middle_pos.x;
        base_circle_transform.translation.y = middle_pos.y;
        base_circle_transform.translation.z = edge_depth_drawn;
        base_circle_transform.scale = Vec3::splat(edge_angle_base_circle_scale);

        let Ok(mut end_circle_transform) = transform_q.get_mut(end_circle_entity) else {
            warn!(
                "Edge angle end circle entity {:?} has no transform",
                end_circle_entity
            );
            return;
        };
        end_circle_transform.translation.x = edge_angle_endpoint.x;
        end_circle_transform.translation.y = edge_angle_endpoint.y;
        end_circle_transform.translation.z = edge_depth_drawn;
        end_circle_transform.scale = Vec3::splat(edge_angle_end_circle_scale);
    }
}

fn get_3d_line_rotation_and_scale(start: Vec3, end: Vec3, spin: f32) -> (Quat, f32) {
    let translation_diff = end - start;
    let target_direction = translation_diff.normalize();

    (
        quat_from_spin_direction(spin, Vec3::X, target_direction),
        start.distance(end),
    )
}

fn get_inversed_final_rotation(
    vertex_manager: &VertexManager,
    anim_manager: &AnimationManager,
    frame_entity: Entity,
    vertex_entity: Entity,
    name_q: &Query<&ShapeName>,
    rotation_q: &Query<(&mut AnimRotation, &LocalAnimRotation)>,
    target_rotation: &mut Quat,
) {
    if let Ok(name_component) = name_q.get(vertex_entity) {
        let name = (*name_component.value).clone();
        if let Some(rotation_entity) = anim_manager.vertex_names.get(&(frame_entity, name)) {
            let (_, anim_rotation) = rotation_q.get(*rotation_entity).unwrap();
            *target_rotation =
                (anim_rotation.last_synced_quat.inverse() * *target_rotation).normalize();
        }
    }

    if let Some(parent_entity) = vertex_manager.vertex_parent_3d_entity(&vertex_entity) {
        get_inversed_final_rotation(
            vertex_manager,
            anim_manager,
            frame_entity,
            parent_entity,
            name_q,
            rotation_q,
            target_rotation,
        );
    }
}

pub fn get_root_vertex(world: &mut World) -> Option<Entity> {
    let mut system_state: SystemState<(
        Query<(Entity, &Vertex3d)>,
        Query<&Visibility>,
        Query<Entity, With<VertexRoot>>,
    )> = SystemState::new(world);
    let (vertex_3d_q, visibility_q, root_q) = system_state.get_mut(world);

    // find root 3d vertex
    let mut root_3d_vertex = None;
    for vertex_3d_entity in root_q.iter() {
        if let Ok(visibility) = visibility_q.get(vertex_3d_entity) {
            if !visibility.visible {
                continue;
            }
        };
        if vertex_3d_q.get(vertex_3d_entity).is_err() {
            continue;
        }
        if root_3d_vertex.is_some() {
            panic!("Multiple root 3d vertices found!");
        }
        root_3d_vertex = Some(vertex_3d_entity);
    }

    root_3d_vertex
}
