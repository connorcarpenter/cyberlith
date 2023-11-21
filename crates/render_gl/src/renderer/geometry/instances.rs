use math::Mat4;

#[derive(Clone, Debug, Default)]
pub struct Instances {
    pub transformations: Vec<Mat4>,
}

impl Instances {
    pub fn new(transforms: Vec<Mat4>) -> Self {
        Self {
            transformations: transforms,
        }
    }

    pub fn count(&self) -> u32 {
        self.transformations.len() as u32
    }
}
