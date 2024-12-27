use math::Vec2;

#[derive(Debug, Clone)]
pub(crate) struct Velocity {
    x: f32,
    y: f32,
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn get_vec2(&self) -> Vec2 {
        // quantize to match what will be serialized
        let x = (self.x * 100.0) as i128;
        let y = (self.y * 100.0) as i128;
        let x = x as f32 / 100.0;
        let y = y as f32 / 100.0;
        Vec2::new(x, y)
    }

    pub fn set_vec2(&mut self, vec2: Vec2) {
        self.x = vec2.x;
        self.y = vec2.y;
    }
}
