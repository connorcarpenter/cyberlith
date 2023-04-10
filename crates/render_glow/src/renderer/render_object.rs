use render_api::{
    base::AxisAlignedBoundingBox,
    components::{Camera, Transform},
};

use crate::{
    core::{ColorTexture, DepthTexture},
    renderer::{BaseMesh, Geometry, Light, Material, MaterialType, Mesh, Object, RenderCamera},
};

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

impl<'a> Object for RenderObject<'a> {
    fn render(&self, camera: &RenderCamera, lights: &[&dyn Light]) {
        self.render_with_material(self.material, camera, lights);
    }

    fn material_type(&self) -> MaterialType {
        self.material.material_type()
    }
}
