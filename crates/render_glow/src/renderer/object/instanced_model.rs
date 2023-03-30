use crate::renderer::*;
use render_api::base::{AxisAlignedBoundingBox, Geometry as CpuGeometry, Model as CpuModel};

///
/// Part of an [InstancedModel] consisting of a [InstancedMesh], some type of [material]
///
pub struct InstancedModelPart<M: Material> {
    gm: Gm<InstancedMesh, M>,
}

impl<M: Material> std::ops::Deref for InstancedModelPart<M> {
    type Target = Gm<InstancedMesh, M>;
    fn deref(&self) -> &Self::Target {
        &self.gm
    }
}

impl<M: Material> std::ops::DerefMut for InstancedModelPart<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.gm
    }
}

impl<M: Material> Geometry for InstancedModelPart<M> {
    fn render_with_material(
        &self,
        material: &dyn Material,
        camera: &Camera,
        lights: &[&dyn Light],
    ) {
        self.gm.render_with_material(material, camera, lights)
    }

    fn render_with_post_material(
        &self,
        material: &dyn PostMaterial,
        camera: &Camera,
        lights: &[&dyn Light],
        color_texture: Option<ColorTexture>,
        depth_texture: Option<DepthTexture>,
    ) {
        self.gm
            .render_with_post_material(material, camera, lights, color_texture, depth_texture)
    }

    fn aabb(&self) -> AxisAlignedBoundingBox {
        self.gm.aabb()
    }
}

impl<M: Material> Object for InstancedModelPart<M> {
    fn render(&self, camera: &Camera, lights: &[&dyn Light]) {
        self.gm.render(camera, lights)
    }

    fn material_type(&self) -> MaterialType {
        self.gm.material_type()
    }
}

impl<'a, M: Material> IntoIterator for &'a InstancedModelPart<M> {
    type Item = &'a dyn Object;
    type IntoIter = std::iter::Once<&'a dyn Object>;

    fn into_iter(self) -> Self::IntoIter {
        self.gm.into_iter()
    }
}

///
/// Similar to [Model], except it is possible to render many instances of the same model efficiently.
///
pub struct InstancedModel<M: Material>(Vec<InstancedModelPart<M>>);

impl<'a, M: Material> IntoIterator for &'a InstancedModel<M> {
    type Item = &'a dyn Object;
    type IntoIter = std::vec::IntoIter<&'a dyn Object>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
            .map(|m| m as &dyn Object)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

impl<M: Material + FromPbrMaterial + Clone + Default> InstancedModel<M> {
    ///
    /// Constructs an [InstancedModel] from a [Model] and the given [Instances] attributes, ie. constructs a list of [Gm]s with a [InstancedMesh] as geometry (constructed from the [TriMesh]es in the [Model]) and
    /// a [material] type specified by the generic parameter which implement [FromPbrMaterial] (constructed from the [PbrMaterial]s in the [Model]).
    ///
    pub fn new(
        context: &Context,
        instances: &Instances,
        cpu_model: &CpuModel,
    ) -> Result<Self, RendererError> {
        let materials = cpu_model
            .materials
            .iter()
            .map(|m| M::from_cpu_material(context, m))
            .collect::<Vec<_>>();
        let mut gms = Vec::new();
        for primitive in cpu_model.geometries.iter() {
            if let CpuGeometry::Triangles(geometry) = &primitive.geometry {
                let material = if let Some(material_index) = primitive.material_index {
                    materials
                        .get(material_index)
                        .ok_or_else(|| {
                            RendererError::MissingMaterial(
                                material_index.to_string(),
                                primitive.name.clone(),
                            )
                        })?
                        .clone()
                } else {
                    M::default()
                };
                let mut gm = Gm {
                    geometry: InstancedMesh::new(context, instances, geometry),
                    material,
                };
                gm.set_transformation(primitive.transformation);
                gms.push(InstancedModelPart { gm });
            }
        }
        let mut model = Self(gms);
        Ok(model)
    }
}

impl<M: Material> std::ops::Deref for InstancedModel<M> {
    type Target = Vec<InstancedModelPart<M>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<M: Material> std::ops::DerefMut for InstancedModel<M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
