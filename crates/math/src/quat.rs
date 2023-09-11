use glam::Quat;

use naia_serde::SerdeInternal as Serde;

#[derive(Clone, Copy, PartialEq)]
pub struct SerdeQuat {
    quat: Quat,
}

impl From<Quat> for SerdeQuat {
    fn from(quat: Quat) -> Self {
        Self {
            quat
        }
    }
}

impl Into<Quat> for SerdeQuat {
    fn into(self) -> Quat {
        self.quat
    }
}

#[derive(Serde, Clone, Copy, PartialEq)]
enum SkipComponent {
    X,
    Y,
    Z,
    W,
}

impl Serde for SerdeQuat {

}

mod tests {
    #[test]
    fn example() {
        assert_eq!(true, true);
        assert_eq!(true, false);
    }
}