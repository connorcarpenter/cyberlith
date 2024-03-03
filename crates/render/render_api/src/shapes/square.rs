use math::Vec3;
use storage::StorageHash;

use crate::base::CpuMesh;

// CenteredSquare
#[derive(Hash)]
pub struct CenteredSquare;

impl CenteredSquare {
    pub fn new() -> Self {
        Self
    }
}

impl StorageHash<CpuMesh> for CenteredSquare {}

impl From<CenteredSquare> for CpuMesh {
    fn from(_square: CenteredSquare) -> Self {
        let half_size = 1.0;
        let neg_half_size = -1.0;

        let positions = vec![
            Vec3::new(neg_half_size, neg_half_size, 0.0),
            Vec3::new(half_size, neg_half_size, 0.0),
            Vec3::new(half_size, half_size, 0.0),
            Vec3::new(neg_half_size, half_size, 0.0),
        ];

        let indices = vec![0, 2, 1, 2, 0, 3];

        CpuMesh::from_indices(&positions, &indices)
    }
}

// UnitSquare
#[derive(Hash)]
pub struct UnitSquare;

impl UnitSquare {
    pub fn new() -> Self {
        Self
    }
}

impl StorageHash<CpuMesh> for UnitSquare {}

impl From<UnitSquare> for CpuMesh {
    fn from(_square: UnitSquare) -> Self {
        let positions = vec![
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(1.0, 1.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
        ];

        let indices = vec![0, 2, 1, 2, 0, 3];

        CpuMesh::from_indices(&positions, &indices)
    }
}

// HollowSquare
#[derive(Hash)]
pub struct HollowRectangle {
    pub width_thousandths: u32,
    pub height_thousandths: u32,
}

impl HollowRectangle {
    pub fn new(width_thousandths: u32, height_thousandths: u32) -> Self {
        Self {
            width_thousandths,
            height_thousandths,
        }
    }
}

impl StorageHash<CpuMesh> for HollowRectangle {}

impl From<HollowRectangle> for CpuMesh {
    fn from(square: HollowRectangle) -> Self {
        let width = square.width_thousandths as f32 / 1000.0;
        let height = square.height_thousandths as f32 / 1000.0;
        let line_thickness = 0.5;

        let outer_half_width = width + line_thickness;
        let outer_neg_half_width = outer_half_width * -1.0;

        let inner_half_width = width - line_thickness;
        let inner_neg_half_width = inner_half_width * -1.0;

        let outer_half_height = height + line_thickness;
        let outer_neg_half_height = outer_half_height * -1.0;

        let inner_half_height = height - line_thickness;
        let inner_neg_half_height = inner_half_height * -1.0;

        let positions = vec![
            Vec3::new(inner_neg_half_width, inner_neg_half_height, 0.0),
            Vec3::new(outer_neg_half_width, outer_neg_half_height, 0.0),
            Vec3::new(inner_half_width, inner_neg_half_height, 0.0),
            Vec3::new(outer_half_width, outer_neg_half_height, 0.0),
            Vec3::new(inner_half_width, inner_half_height, 0.0),
            Vec3::new(outer_half_width, outer_half_height, 0.0),
            Vec3::new(inner_neg_half_width, inner_half_height, 0.0),
            Vec3::new(outer_neg_half_width, outer_half_height, 0.0),
        ];

        let mut indices = Vec::new();

        for j in 0u16..4 {
            let a = j * 2;
            let b = j * 2 + 1;
            let i = (j + 1) % 4;
            let c = i * 2;
            let d = i * 2 + 1;

            indices.push(a as usize);
            indices.push(b as usize);
            indices.push(c as usize);

            indices.push(c as usize);
            indices.push(b as usize);
            indices.push(d as usize);
        }

        CpuMesh::from_indices(&positions, &indices)
    }
}
