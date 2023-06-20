use std::{collections::HashMap, sync::RwLock};

use math::{Mat3, Mat4, Vec3};
use render_api::base::{AxisAlignedBoundingBox, Color};
use render_api::components::CameraProjection;

use crate::{core::*, renderer::*};

use super::GpuMesh;

///
/// Similar to [Mesh], except it is possible to render many instances of the same mesh efficiently.
///
pub struct InstancedMesh<'a> {
    gpu_mesh: &'a GpuMesh,
    aabb: AxisAlignedBoundingBox,
    instances: &'a Instances,
    instance_buffers: RwLock<(HashMap<String, InstanceBuffer>, Vec3)>,
}

impl<'a> InstancedMesh<'a> {
    pub fn compose(gpu_mesh: &'a GpuMesh, instances: &'a Instances) -> Self {
        let aabb = gpu_mesh.aabb;
        let mut new_mesh = Self {
            gpu_mesh,
            instances,
            instance_buffers: RwLock::new((Default::default(), Vec3::ZERO)),
            aabb,
        };

        new_mesh.reset_instances();
        new_mesh
    }

    /// Returns the number of instances that is rendered.
    fn instance_count(&self) -> u32 {
        self.instances.transformations.len() as u32
    }

    ///
    /// Reset the instances.
    ///
    fn reset_instances(&mut self) {
        #[cfg(debug_assertions)]
        self.instances.validate().expect("invalid instances");
        self.update_aabb();

        {
            let mut s = self
                .instance_buffers
                .write()
                .expect("failed acquiring write accesss");
            s.0.clear();
        }
    }

    fn update_aabb(&mut self) {
        let mut aabb = AxisAlignedBoundingBox::EMPTY;
        for i in 0..self.instance_count() as usize {
            let mut aabb2 = self.gpu_mesh.aabb;
            aabb2.transform(&(self.instances.transformations[i]));
            aabb.expand_with_aabb(&aabb2);
        }
        self.aabb = aabb;
    }

    /// Update the instance buffers
    fn update_instance_buffers(&self) {
        let needs_update = {
            let s = self
                .instance_buffers
                .read()
                .expect("failed acquiring read accesss");

            // Update is always needed if the instance buffers is empty; Opaque materials only.
            s.0.is_empty()
        };

        if needs_update {
            let mut s = self
                .instance_buffers
                .write()
                .expect("failed acquiring mutable access");
            s.0 = self.create_instance_buffers();
        }
    }

    ///
    /// This function creates the instance buffers, ordering them by distance to the camera
    ///
    fn create_instance_buffers(&self) -> HashMap<String, InstanceBuffer> {
        let indices = {
            // No need to order, just return the indices as is.
            (0..self.instances.transformations.len()).collect::<Vec<usize>>()
        };

        // Next, we can compute the instance buffers with that ordering.
        let mut instance_buffers: HashMap<String, InstanceBuffer> = Default::default();

        // this is checking whether or not any rotations or scaling is applied to any instance
        // this is a pretty nice approach, I can imagine that this is a common case
        if indices
            .iter()
            .map(|i| self.instances.transformations[*i])
            .all(|t| {
                Mat3::from_cols(
                    t.x_axis.truncate(),
                    t.y_axis.truncate(),
                    t.z_axis.truncate(),
                ) == Mat3::IDENTITY
            })
        {
            // if there is no rotation or scaling, just use "instance_translation" to store the position
            instance_buffers.insert(
                "instance_translation".to_string(),
                InstanceBuffer::new_with_data(
                    &indices
                        .iter()
                        .map(|i| self.instances.transformations[*i])
                        .map(|t| t.w_axis.truncate())
                        .collect::<Vec<_>>(),
                ),
            );
        } else {
            let mut row1 = Vec::new();
            let mut row2 = Vec::new();
            let mut row3 = Vec::new();
            for transformation in indices.iter().map(|i| self.instances.transformations[*i]) {
                row1.push(transformation.row(0));
                row2.push(transformation.row(1));
                row3.push(transformation.row(2));
            }

            instance_buffers.insert("row1".to_string(), InstanceBuffer::new_with_data(&row1));
            instance_buffers.insert("row2".to_string(), InstanceBuffer::new_with_data(&row2));
            instance_buffers.insert("row3".to_string(), InstanceBuffer::new_with_data(&row3));
        }

        if let Some(texture_transforms) = &self.instances.texture_transformations {
            let mut instance_tex_transform1 = Vec::new();
            let mut instance_tex_transform2 = Vec::new();
            for texture_transform in indices.iter().map(|i| texture_transforms[*i]) {
                instance_tex_transform1.push(Vec3::new(
                    texture_transform.x_axis.x,
                    texture_transform.y_axis.x,
                    texture_transform.z_axis.x,
                ));
                instance_tex_transform2.push(Vec3::new(
                    texture_transform.x_axis.y,
                    texture_transform.y_axis.y,
                    texture_transform.z_axis.y,
                ));
            }
            instance_buffers.insert(
                "tex_transform_row1".to_string(),
                InstanceBuffer::new_with_data(&instance_tex_transform1),
            );
            instance_buffers.insert(
                "tex_transform_row2".to_string(),
                InstanceBuffer::new_with_data(&instance_tex_transform2),
            );
        }
        if let Some(instance_colors) = &self.instances.colors {
            // Create the re-ordered color buffer by depth.
            let ordered_instance_colors = indices
                .iter()
                .map(|i| instance_colors[*i])
                .collect::<Vec<Color>>();
            instance_buffers.insert(
                "instance_color".to_string(),
                InstanceBuffer::new_with_data(&ordered_instance_colors),
            );
        }
        instance_buffers
    }

    fn draw(
        &self,
        program: &Program,
        render_states: RenderStates,
        render_camera: &'a RenderCamera<'a>,
        attributes: FragmentAttributes,
        instance_buffers: &HashMap<String, InstanceBuffer>,
    ) {
        let camera = render_camera.camera;
        let transform = Mat4::IDENTITY;

        if attributes.normal && instance_buffers.contains_key("instance_translation") {
            let inverse = transform.inverse();
            program.use_uniform("normalMatrix", inverse.transpose());
        }
        program.use_uniform(
            "viewProjection",
            render_camera
                .projection
                .projection_matrix(&camera.viewport_or_default())
                * render_camera.transform.view_matrix(),
        );
        program.use_uniform("modelMatrix", transform);

        for attribute_name in [
            "instance_translation",
            "row1",
            "row2",
            "row3",
            "tex_transform_row1",
            "tex_transform_row2",
            "instance_color",
        ] {
            if program.requires_attribute(attribute_name) {
                program.use_instance_attribute(
                    attribute_name,
                    instance_buffers
                        .get(attribute_name).unwrap_or_else(|| panic!("the render call requires the {} instance buffer which is missing on the given geometry", attribute_name)),
                );
            }
        }
        self.gpu_mesh.draw_instanced(
            program,
            render_states,
            camera,
            attributes,
            self.instance_count(),
        );
    }

    fn vertex_shader_source(
        &self,
        required_attributes: FragmentAttributes,
        instance_buffers: &HashMap<String, InstanceBuffer>,
    ) -> String {
        format!(
            "{}{}{}{}{}{}{}",
            if instance_buffers.contains_key("instance_translation") {
                "#define USE_INSTANCE_TRANSLATIONS\n"
            } else {
                "#define USE_INSTANCE_TRANSFORMS\n"
            },
            if required_attributes.normal {
                "#define USE_NORMALS\n"
            } else {
                ""
            },
            if required_attributes.uv {
                "#define USE_UVS\n"
            } else {
                ""
            },
            if instance_buffers.contains_key("instance_color") && self.gpu_mesh.colors.is_some() {
                "#define USE_VERTEX_COLORS\n#define USE_INSTANCE_COLORS\n"
            } else if instance_buffers.contains_key("instance_color") {
                "#define USE_INSTANCE_COLORS\n"
            } else if self.gpu_mesh.colors.is_some() {
                "#define USE_VERTEX_COLORS\n"
            } else {
                ""
            },
            if instance_buffers.contains_key("tex_transform_row1") {
                "#define USE_INSTANCE_TEXTURE_TRANSFORMATION\n"
            } else {
                ""
            },
            include_str!("../../core/shared.frag"),
            include_str!("shaders/mesh.vert"),
        )
    }

    pub fn aabb(&self) -> AxisAlignedBoundingBox {
        self.aabb
    }

    pub fn render_with_material(
        &self,
        material: &dyn Material,
        render_camera: &RenderCamera,
        lights: &[&dyn Light],
    ) {
        self.update_instance_buffers();
        let instance_buffers = &self
            .instance_buffers
            .read()
            .expect("failed to acquire read access")
            .0;

        let fragment_shader = material.fragment_shader(lights);
        let vertex_shader_source =
            self.vertex_shader_source(fragment_shader.attributes, instance_buffers);
        Context::get()
            .program(vertex_shader_source, fragment_shader.source, |program| {
                material.use_uniforms(program, render_camera, lights);
                self.draw(
                    program,
                    material.render_states(),
                    render_camera,
                    fragment_shader.attributes,
                    instance_buffers,
                );
            })
            .expect("Failed compiling shader");
    }
}

///
/// Defines the attributes for the instances of the model defined in [InstancedMesh] or [InstancedModel].
///
/// Each list of attributes must contain the same number of elements as the number of instances.
/// The attributes are applied to each instance before they are rendered.
/// The [Instances::transformations] are applied after the transformation applied to all instances (see [InstancedMesh::set_transformation]).
///
#[derive(Clone, Debug, Default)]
pub struct Instances {
    /// The transformations applied to each instance.
    pub transformations: Vec<Mat4>,
    /// The texture transform applied to the uv coordinates of each instance.
    pub texture_transformations: Option<Vec<Mat3>>,
    /// Colors multiplied onto the base color of each instance.
    pub colors: Option<Vec<Color>>,
}

impl Instances {
    pub fn new(transforms: Vec<Mat4>) -> Self {
        Self {
            transformations: transforms,
            texture_transformations: None,
            colors: None,
        }
    }

    ///
    /// Returns an error if the instances is not valid.
    ///
    pub fn validate(&self) -> Result<(), RendererError> {
        let instance_count = self.count();
        let buffer_check = |length: Option<usize>, name: &str| -> Result<(), RendererError> {
            if let Some(length) = length {
                if length < instance_count as usize {
                    Err(RendererError::InvalidBufferLength(
                        name.to_string(),
                        instance_count as usize,
                        length,
                    ))?;
                }
            }
            Ok(())
        };

        buffer_check(
            self.texture_transformations.as_ref().map(|b| b.len()),
            "texture transformations",
        )?;
        buffer_check(Some(self.transformations.len()), "transformations")?;
        buffer_check(self.colors.as_ref().map(|b| b.len()), "colors")?;

        Ok(())
    }

    /// Returns the number of instances.
    pub fn count(&self) -> u32 {
        self.transformations.len() as u32
    }
}
