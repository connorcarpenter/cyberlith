use asset_serde::bits::MovementConfigBits;

pub struct MovementConfigData {
    max_velocity: f32,
}

impl Default for MovementConfigData {
    fn default() -> Self {
        panic!("");
    }
}

impl From<&[u8]> for MovementConfigData {
    fn from(bytes: &[u8]) -> Self {
        // info!("--- reading movement config ---");

        let base = MovementConfigBits::from_bytes(bytes).expect("unable to parse file");

        // info!("--- done reading movement config ---");

        Self {
            max_velocity: base.get_max_velocity(),
        }
    }
}

impl MovementConfigData {
    pub fn get_max_velocity(&self) -> f32 {
        self.max_velocity
    }
}
