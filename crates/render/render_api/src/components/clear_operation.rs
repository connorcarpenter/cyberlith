// Clear Operation
#[derive(Clone, Copy)]
pub struct ClearOperation {
    pub red: Option<f32>,
    pub green: Option<f32>,
    pub blue: Option<f32>,
    pub alpha: Option<f32>,
    pub depth: Option<f32>,
}

impl ClearOperation {
    pub fn from_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            red: Some(r),
            green: Some(g),
            blue: Some(b),
            alpha: Some(a),
            depth: Some(1.0),
        }
    }

    pub const fn none() -> Self {
        Self {
            red: None,
            green: None,
            blue: None,
            alpha: None,
            depth: None,
        }
    }
}

impl Default for ClearOperation {
    fn default() -> Self {
        Self {
            red: Some(0.0),
            green: Some(0.0),
            blue: Some(0.0),
            alpha: Some(1.0),
            depth: Some(1.0),
        }
    }
}
