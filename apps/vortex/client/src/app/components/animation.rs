// LocalAnimRotation
#[derive(Component)]
pub struct LocalAnimRotation {
    pub frame_entity: Entity,
    pub vertex_3d_entity: Entity,
}

impl AnimRotation {
    pub fn new(frame_entity: Entity, vertex_3d_entity: Entity) -> Self {
        Self {
            frame_entity,
            vertex_3d_entity,
        }
    }
}