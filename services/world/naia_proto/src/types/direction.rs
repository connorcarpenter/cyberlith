use naia_bevy_shared::Serde;

use random::gen_range_u32;

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
    pub(crate) fn congruent_with(&self, other: Direction) -> bool {
        match self {
            Direction::North => matches!(
                other,
                Direction::North | Direction::Northeast | Direction::Northwest
            ),
            Direction::Northeast => matches!(
                other,
                Direction::Northeast | Direction::East | Direction::North
            ),
            Direction::East => matches!(
                other,
                Direction::East | Direction::Southeast | Direction::Northeast
            ),
            Direction::Southeast => matches!(
                other,
                Direction::Southeast | Direction::South | Direction::East
            ),
            Direction::South => matches!(
                other,
                Direction::South | Direction::Southeast | Direction::Southwest
            ),
            Direction::Southwest => matches!(
                other,
                Direction::Southwest | Direction::South | Direction::West
            ),
            Direction::West => matches!(
                other,
                Direction::West | Direction::Southwest | Direction::Northwest
            ),
            Direction::Northwest => matches!(
                other,
                Direction::Northwest | Direction::North | Direction::West
            ),
        }
    }
}

impl Direction {
    pub fn random() -> Self {
        let i = gen_range_u32(0, 8);
        match i {
            0 => Direction::North,
            1 => Direction::Northeast,
            2 => Direction::East,
            3 => Direction::Southeast,
            4 => Direction::South,
            5 => Direction::Southwest,
            6 => Direction::West,
            7 => Direction::Northwest,
            _ => panic!("Invalid random direction"),
        }
    }

    pub fn from_radians(angle: f32) -> Self {
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

    pub fn from_coords(x: f32, y: f32) -> Self {
        let angle = (y * -1.0).atan2(x);
        Self::from_radians(angle)
    }

    pub fn to_delta(&self) -> (i8, i8) {
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

    pub fn from_delta(dx: i8, dy: i8) -> Result<Option<Self>, ()> {
        match (dx, dy) {
            (0, -1) => Ok(Some(Direction::North)),
            (1, -1) => Ok(Some(Direction::Northeast)),
            (1, 0) => Ok(Some(Direction::East)),
            (1, 1) => Ok(Some(Direction::Southeast)),
            (0, 1) => Ok(Some(Direction::South)),
            (-1, 1) => Ok(Some(Direction::Southwest)),
            (-1, 0) => Ok(Some(Direction::West)),
            (-1, -1) => Ok(Some(Direction::Northwest)),
            (0, 0) => Ok(None),
            _ => Err(()),
        }
    }

    pub fn to_opposite(&self) -> Self {
        let (dx, dy) = self.to_delta();
        Direction::from_delta(-dx, -dy).unwrap().unwrap()
    }

    pub fn to_radians(&self) -> f32 {
        match self {
            Direction::North => 270.0_f32.to_radians(),
            Direction::Northeast => 315.0_f32.to_radians(),
            Direction::East => 0.0_f32.to_radians(),
            Direction::Southeast => 45.0_f32.to_radians(),
            Direction::South => 90.0_f32.to_radians(),
            Direction::Southwest => 135.0_f32.to_radians(),
            Direction::West => 180.0_f32.to_radians(),
            Direction::Northwest => 225.0_f32.to_radians(),
        }
    }
}

mod tests {
    #[test]
    fn from_angle_tests() {
        use super::Direction;
        assert_eq!(
            Direction::from_radians(0.0_f32.to_radians()),
            Direction::East
        );
        assert_eq!(
            Direction::from_radians(45.0_f32.to_radians()),
            Direction::Northeast
        );
        assert_eq!(
            Direction::from_radians(90.0_f32.to_radians()),
            Direction::North
        );
        assert_eq!(
            Direction::from_radians(135.0_f32.to_radians()),
            Direction::Northwest
        );
        assert_eq!(
            Direction::from_radians(180.0_f32.to_radians()),
            Direction::West
        );
        assert_eq!(
            Direction::from_radians(225.0_f32.to_radians()),
            Direction::Southwest
        );
        assert_eq!(
            Direction::from_radians(270.0_f32.to_radians()),
            Direction::South
        );
        assert_eq!(
            Direction::from_radians(315.0_f32.to_radians()),
            Direction::Southeast
        );
        assert_eq!(
            Direction::from_radians(360.0_f32.to_radians()),
            Direction::East
        );
        assert_eq!(
            Direction::from_radians(-45.0_f32.to_radians()),
            Direction::Southeast
        );
        assert_eq!(
            Direction::from_radians(-90.0_f32.to_radians()),
            Direction::South
        );
        assert_eq!(
            Direction::from_radians(-135.0_f32.to_radians()),
            Direction::Southwest
        );
        assert_eq!(
            Direction::from_radians(-180.0_f32.to_radians()),
            Direction::West
        );
        assert_eq!(
            Direction::from_radians(-225.0_f32.to_radians()),
            Direction::Northwest
        );
        assert_eq!(
            Direction::from_radians(-270.0_f32.to_radians()),
            Direction::North
        );
        assert_eq!(
            Direction::from_radians(-315.0_f32.to_radians()),
            Direction::Northeast
        );
        assert_eq!(
            Direction::from_radians(-360.0_f32.to_radians()),
            Direction::East
        );

        // test boundary conditions
        assert_eq!(
            Direction::from_radians(22.4_f32.to_radians()),
            Direction::East
        );
        assert_eq!(
            Direction::from_radians(22.6_f32.to_radians()),
            Direction::Northeast
        );
        assert_eq!(
            Direction::from_radians(67.4_f32.to_radians()),
            Direction::Northeast
        );
        assert_eq!(
            Direction::from_radians(67.6_f32.to_radians()),
            Direction::North
        );
        assert_eq!(
            Direction::from_radians(112.4_f32.to_radians()),
            Direction::North
        );
        assert_eq!(
            Direction::from_radians(112.6_f32.to_radians()),
            Direction::Northwest
        );
        assert_eq!(
            Direction::from_radians(157.4_f32.to_radians()),
            Direction::Northwest
        );
        assert_eq!(
            Direction::from_radians(157.6_f32.to_radians()),
            Direction::West
        );
        assert_eq!(
            Direction::from_radians(202.4_f32.to_radians()),
            Direction::West
        );
        assert_eq!(
            Direction::from_radians(202.6_f32.to_radians()),
            Direction::Southwest
        );
        assert_eq!(
            Direction::from_radians(247.4_f32.to_radians()),
            Direction::Southwest
        );
        assert_eq!(
            Direction::from_radians(247.6_f32.to_radians()),
            Direction::South
        );
        assert_eq!(
            Direction::from_radians(292.4_f32.to_radians()),
            Direction::South
        );
        assert_eq!(
            Direction::from_radians(292.6_f32.to_radians()),
            Direction::Southeast
        );
        assert_eq!(
            Direction::from_radians(337.4_f32.to_radians()),
            Direction::Southeast
        );
        assert_eq!(
            Direction::from_radians(337.6_f32.to_radians()),
            Direction::East
        );

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
