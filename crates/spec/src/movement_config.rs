pub struct MovementConfig {
    max_velocity: f32,
}

impl MovementConfig {
    pub fn new(max_velocity: f32) -> Self {
        Self {
            max_velocity,
        }
    }

    pub fn max_velocity(&self) -> f32 {
        self.max_velocity
    }
}