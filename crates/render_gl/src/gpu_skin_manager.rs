use std::{collections::HashMap, default::Default};

use bevy_ecs::system::Resource;

use render_api::{base::CpuSkin, Handle};

use crate::{core::{GpuTexture2D, Program}, GpuMaterialManager, renderer::FragmentShader};

#[derive(Resource)]
pub struct GpuSkinManager {
    assets: HashMap<Handle<CpuSkin>, GpuSkin>,
    gpu_skins: Option<GpuTexture2D>,
    cpu_skins: Vec<CpuSkin>,
    biggest_skin: usize,
}

impl Default for GpuSkinManager {
    fn default() -> Self {
        Self {
            assets: HashMap::new(),
            gpu_skins: None,
            cpu_skins: Vec::new(),
            biggest_skin: 0,
        }
    }
}

impl GpuSkinManager {
    pub fn insert(&mut self, gpu_material_manager: &GpuMaterialManager, handle: Handle<CpuSkin>, cpu_skin: &CpuSkin) {
        let new_index = self.cpu_skins.len();
        let gpu_skin = GpuSkin::new(new_index);
        self.assets.insert(handle, gpu_skin);

        self.cpu_skins.push(cpu_skin.clone());

        let skin_size = cpu_skin.len();
        if skin_size > self.biggest_skin {
            self.biggest_skin = skin_size;
        }

        self.gpu_sync(gpu_material_manager);
    }

    fn gpu_sync(&mut self, gpu_material_manager: &GpuMaterialManager) {
        self.gpu_skins = Some(GpuTexture2D::new_empty::<f32>(
            self.biggest_skin as u32,
            self.cpu_skins.len() as u32,
        ));

        let skin_data = self.raw_skin_data(gpu_material_manager);

        let gpu_materials = self.gpu_skins.as_mut().unwrap();
        gpu_materials.fill_pure(&skin_data);
    }

    fn raw_skin_data(&self, gpu_material_manager: &GpuMaterialManager) -> Vec<f32> {
        let mut output = Vec::new();
        for skin in &self.cpu_skins {
            write_raw_data(gpu_material_manager,  skin, self.biggest_skin, &mut output);
        }
        output
    }

    pub fn get(&self, handle: &Handle<CpuSkin>) -> Option<&GpuSkin> {
        self.assets.get(&handle)
    }

    pub fn remove(&mut self, _handle: &Handle<CpuSkin>) -> Option<GpuSkin> {
        todo!();
        self.assets.remove(_handle)
    }

    pub fn fragment_shader(&self) -> FragmentShader {
        let output = include_str!("shaders/physical_material.frag").to_string();
        FragmentShader {
            source: output,
        }
    }

    pub fn use_uniforms(&self, program: &Program) {
        program.use_texture("skin_texture", self.gpu_skins.as_ref().unwrap());
    }

    pub fn is_ready(&self) -> bool {
        self.gpu_skins.is_some()
    }

    pub fn get_ready(&mut self) {
        if self.gpu_skins.is_some() {
            return;
        }
        self.gpu_skins = Some(GpuTexture2D::new_empty::<f32>(
            0,
            0,
        ));
    }
}

fn write_raw_data(gpu_material_manager: &GpuMaterialManager, skin: &CpuSkin, biggest_skin: usize, output: &mut Vec<f32>) {
    //info!("write_raw_data() .. biggest_skin: {}", biggest_skin);

    let mut temp = Vec::new();

    let skin_list = skin.face_to_material_list();
    for mat_handle in skin_list {
        let mat = gpu_material_manager.get(mat_handle).unwrap();
        let index = mat.index();
        temp.push(index as f32);
    }
    for _ in temp.len()..biggest_skin {
        temp.push(0.0);
    }

    //info!("data size: {}", temp.len());

    output.extend(temp);
}

pub struct GpuSkin {
    // the index of the GpuSkin is the y coordinate of the pixel in the SkinTexture
    index: usize,
}

impl GpuSkin {
    pub fn new(index: usize) -> Self {
        Self { index }
    }

    pub fn index(&self) -> usize {
        self.index
    }
}
