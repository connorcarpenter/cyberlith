use naia_bevy_shared::Serde;

#[derive(Serde, Clone, Debug, Copy, Eq, PartialEq)]
pub enum Direction {
    North,
    Northeast,
    East,
    Southeast,
    South,
    Southwest,
    West,
    Northwest,
}

impl Direction {
    pub fn from_angle(angle: f32) -> Self {
        let angle = angle + std::f32::consts::PI / 8.0;
        let angle = if angle < 0.0 {
            angle + 2.0 * std::f32::consts::PI
        } else {
            angle
        };
        let angle = angle % (2.0 * std::f32::consts::PI);
        let angle = angle / (std::f32::consts::PI / 4.0);
        match angle as u32 {
            0 => Direction::East,
            1 => Direction::Northeast,
            2 => Direction::North,
            3 => Direction::Northwest,
            4 => Direction::West,
            5 => Direction::Southwest,
            6 => Direction::South,
            7 => Direction::Southeast,
            _ => Direction::East,
        }
    }

    pub(crate) fn from_coords(x: f32, y: f32) -> Self {
        let angle = (y * -1.0).atan2(x);
        Self::from_angle(angle)
    }

    pub(crate) fn to_delta(&self) -> (i8, i8) {
        match self {
            Direction::North => (0, -1),
            Direction::Northeast => (1, -1),
            Direction::East => (1, 0),
            Direction::Southeast => (1, 1),
            Direction::South => (0, 1),
            Direction::Southwest => (-1, 1),
            Direction::West => (-1, 0),
            Direction::Northwest => (-1, -1),
        }
    }

    pub(crate) fn from_delta(dx: i8, dy: i8) -> Option<Self> {
        match (dx, dy) {
            (0, -1) => Some(Direction::North),
            (1, -1) => Some(Direction::Northeast),
            (1, 0) => Some(Direction::East),
            (1, 1) => Some(Direction::Southeast),
            (0, 1) => Some(Direction::South),
            (-1, 1) => Some(Direction::Southwest),
            (-1, 0) => Some(Direction::West),
            (-1, -1) => Some(Direction::Northwest),
            _ => None,
        }
    }
}

mod tests {
    #[test]
    fn from_angle_tests() {
        use super::Direction;
        assert_eq!(Direction::from_angle(0.0_f32.to_radians()), Direction::East);
        assert_eq!(Direction::from_angle(45.0_f32.to_radians()), Direction::Northeast);
        assert_eq!(Direction::from_angle(90.0_f32.to_radians()), Direction::North);
        assert_eq!(Direction::from_angle(135.0_f32.to_radians()), Direction::Northwest);
        assert_eq!(Direction::from_angle(180.0_f32.to_radians()), Direction::West);
        assert_eq!(Direction::from_angle(225.0_f32.to_radians()), Direction::Southwest);
        assert_eq!(Direction::from_angle(270.0_f32.to_radians()), Direction::South);
        assert_eq!(Direction::from_angle(315.0_f32.to_radians()), Direction::Southeast);
        assert_eq!(Direction::from_angle(360.0_f32.to_radians()), Direction::East);
        assert_eq!(Direction::from_angle(-45.0_f32.to_radians()), Direction::Southeast);
        assert_eq!(Direction::from_angle(-90.0_f32.to_radians()), Direction::South);
        assert_eq!(Direction::from_angle(-135.0_f32.to_radians()), Direction::Southwest);
        assert_eq!(Direction::from_angle(-180.0_f32.to_radians()), Direction::West);
        assert_eq!(Direction::from_angle(-225.0_f32.to_radians()), Direction::Northwest);
        assert_eq!(Direction::from_angle(-270.0_f32.to_radians()), Direction::North);
        assert_eq!(Direction::from_angle(-315.0_f32.to_radians()), Direction::Northeast);
        assert_eq!(Direction::from_angle(-360.0_f32.to_radians()), Direction::East);

        // test boundary conditions
        assert_eq!(Direction::from_angle(22.4_f32.to_radians()), Direction::East);
        assert_eq!(Direction::from_angle(22.6_f32.to_radians()), Direction::Northeast);
        assert_eq!(Direction::from_angle(67.4_f32.to_radians()), Direction::Northeast);
        assert_eq!(Direction::from_angle(67.6_f32.to_radians()), Direction::North);
        assert_eq!(Direction::from_angle(112.4_f32.to_radians()), Direction::North);
        assert_eq!(Direction::from_angle(112.6_f32.to_radians()), Direction::Northwest);
        assert_eq!(Direction::from_angle(157.4_f32.to_radians()), Direction::Northwest);
        assert_eq!(Direction::from_angle(157.6_f32.to_radians()), Direction::West);
        assert_eq!(Direction::from_angle(202.4_f32.to_radians()), Direction::West);
        assert_eq!(Direction::from_angle(202.6_f32.to_radians()), Direction::Southwest);
        assert_eq!(Direction::from_angle(247.4_f32.to_radians()), Direction::Southwest);
        assert_eq!(Direction::from_angle(247.6_f32.to_radians()), Direction::South);
        assert_eq!(Direction::from_angle(292.4_f32.to_radians()), Direction::South);
        assert_eq!(Direction::from_angle(292.6_f32.to_radians()), Direction::Southeast);
        assert_eq!(Direction::from_angle(337.4_f32.to_radians()), Direction::Southeast);
        assert_eq!(Direction::from_angle(337.6_f32.to_radians()), Direction::East);

        // test coords
        assert_eq!(Direction::from_coords(0.0, -10.0), Direction::North);
        assert_eq!(Direction::from_coords(10.0, -10.0), Direction::Northeast);
        assert_eq!(Direction::from_coords(10.0, 0.0), Direction::East);
        assert_eq!(Direction::from_coords(10.0, 10.0), Direction::Southeast);
        assert_eq!(Direction::from_coords(0.0, 10.0), Direction::South);
        assert_eq!(Direction::from_coords(-10.0, 10.0), Direction::Southwest);
        assert_eq!(Direction::from_coords(-10.0, 0.0), Direction::West);
        assert_eq!(Direction::from_coords(-10.0, -10.0), Direction::Northwest);

        // test coord boundary conditions
        assert_eq!(Direction::from_coords(2.0, -10.0), Direction::North);
        assert_eq!(Direction::from_coords(10.0, -8.0), Direction::Northeast);
    }
}