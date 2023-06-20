use render_api::{base::AxisAlignedBoundingBox, components::Transform};

use crate::renderer::{BaseMesh, Geometry, Light, Material, Mesh, RenderCamera};

// Render Object
#[derive(Clone, Copy)]
pub struct RenderObject<'a> {
    pub mesh: &'a BaseMesh,
    pub material: &'a dyn Material,
    pub transform: &'a Transform,
}

impl<'a> RenderObject<'a> {
    pub fn new(mesh: &'a BaseMesh, material: &'a dyn Material, transform: &'a Transform) -> Self {
        Self {
            mesh,
            material,
            transform,
        }
    }

    pub fn render(&self, camera: &RenderCamera, lights: &[&dyn Light]) {
        self.render_with_material(self.material, camera, lights);
    }
}

impl<'a> Geometry for RenderObject<'a> {
    fn render_with_material(
        &self,
        material: &dyn Material,
        camera: &RenderCamera,
        lights: &[&dyn Light],
    ) {
        let mesh = Mesh::compose(self.mesh, self.transform);
        mesh.render_with_material(material, camera, lights);
    }

    fn aabb(&self) -> AxisAlignedBoundingBox {
        self.mesh.aabb
    }
}
