use std::{collections::HashMap, default::Default};

use bevy_ecs::system::Resource;

use render_api::{base::CpuMaterial, Handle};

use crate::{
    core::{Cull, GpuTexture2D, Program, RenderStates},
    renderer::{FragmentAttributes, FragmentShader, Light, RenderCamera},
};

#[derive(Resource)]
pub struct GpuMaterialManager {
    assets: HashMap<Handle<CpuMaterial>, GpuMaterial>,
    gpu_materials: Option<GpuTexture2D>,
    cpu_materials: Vec<[f32; 4]>,
    render_states: RenderStates,
}

impl Default for GpuMaterialManager {
    fn default() -> Self {
        Self {
            assets: HashMap::new(),
            gpu_materials: None,
            cpu_materials: Vec::new(),
            render_states: RenderStates {
                cull: Cull::Back,
                ..Default::default()
            },
        }
    }
}

impl GpuMaterialManager {
    pub fn insert(&mut self, handle: Handle<CpuMaterial>, cpu_material: &CpuMaterial) {
        let new_index = self.cpu_materials.len();
        let gpu_material = GpuMaterial::new(new_index / 2);
        self.assets.insert(handle, gpu_material);

        let first = [
            cpu_material.diffuse.r as f32 / 255.0,
            cpu_material.diffuse.g as f32 / 255.0,
            cpu_material.diffuse.b as f32 / 255.0,
            cpu_material.emissive,
        ];
        let second = [cpu_material.shine_size, cpu_material.shine_amount, 0.0, 0.0];
        self.cpu_materials.push(first);
        self.cpu_materials.push(second);

        self.gpu_sync();
    }

    fn gpu_sync(&mut self) {
        self.gpu_materials = Some(GpuTexture2D::new_empty::<[f32; 4]>(
            self.cpu_materials.len() as u32,
            1,
        ));
        let gpu_materials = self.gpu_materials.as_mut().unwrap();
        gpu_materials.fill_pure(&self.cpu_materials);
    }

    pub fn get(&self, handle: &Handle<CpuMaterial>) -> Option<&GpuMaterial> {
        self.assets.get(&handle)
    }

    pub fn remove(&mut self, handle: &Handle<CpuMaterial>) -> Option<GpuMaterial> {
        todo!();
        self.assets.remove(handle)
    }

    pub fn fragment_shader(&self) -> FragmentShader {
        let output = include_str!("shaders/physical_material.frag").to_string();
        FragmentShader {
            source: output,
        }
    }

    pub fn use_uniforms(&self, program: &Program, camera: &RenderCamera, lights: &[&dyn Light]) {
        if !lights.is_empty() {
            for (i, light) in lights.iter().enumerate() {
                light.use_uniforms(program, i as u32);
            }
        }
        program.use_uniform_if_required("camera_position", camera.transform.translation);

        //program.use_uniform("material_texture_width", self.cpu_materials.len() as f32);

        program.use_texture("material_texture", self.gpu_materials.as_ref().unwrap());
    }

    pub fn render_states(&self) -> RenderStates {
        self.render_states
    }
}

pub struct GpuMaterial {
    index: usize,
}

impl GpuMaterial {
    pub fn new(index: usize) -> Self {
        Self { index }
    }

    pub fn index(&self) -> usize {
        self.index
    }
}
