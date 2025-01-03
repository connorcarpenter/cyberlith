pub const TILE_SIZE: f32 = 100.0; // should be 100.0
pub const TILE_COUNT: i32 = 5; // should be 5

pub const MOVEMENT_VELOCITY_MAX: f32 = 12.0; // should be 8.0?
pub const MOVEMENT_VELOCITY_MIN: f32 = 3.0;
pub const MOVEMENT_ACCELERATION: f32 = 1.0;
pub const MOVEMENT_FRICTION: f32 = 1.0;
pub const MOVEMENT_ARRIVAL_DISTANCE: f32 = 5.0;
pub const MOVEMENT_STEERING_DEADZONE: f32 = 4.0;
pub const MOVEMENT_INTERMEDIATE_ARRIVAL_DISTANCE: f32 = 5.0;
pub const MISPREDICTION_CORRECTION_DURATION_MS: u64 = 250;
pub const MISPREDICTION_CORRECTION_FACTOR: f32 = 0.01; // 0.0 < X < 1.0, higher is smoother
