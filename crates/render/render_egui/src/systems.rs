use bevy_ecs::{
    system::{NonSendMut, Res, ResMut},
    world::World,
};

use render_api::base::CpuTexture2D;
use render_gl::{FrameInput, GpuTexture2D};
use storage::SideStorage;

use crate::{EguiContext, EguiUserTextures, GUI};

pub fn startup(world: &mut World) {
    world.insert_non_send_resource(GUI::default());
}

pub fn pre_update(
    mut frame_input: NonSendMut<FrameInput>,
    mut gui: NonSendMut<GUI>,
    egui_context: Res<EguiContext>,
) {
    gui.pre_update(egui_context.inner(), frame_input.as_mut());
}

pub fn post_update(
    mut frame_input: NonSendMut<FrameInput>,
    mut gui: NonSendMut<GUI>,
    egui_context: Res<EguiContext>,
) {
    gui.post_update(egui_context.inner(), frame_input.as_mut());
}

pub fn draw(
    //frame_input: NonSendMut<FrameInput>,
    gui: NonSendMut<GUI>,
    egui_context: Res<EguiContext>,
) {
    gui.render(egui_context.inner());
}

pub fn sync(
    mut gui: NonSendMut<GUI>,
    mut user_textures: ResMut<EguiUserTextures>,
    asset_mapping: ResMut<SideStorage<CpuTexture2D, GpuTexture2D>>,
) {
    if !user_textures.must_process() {
        return;
    }

    {
        // Sync Added Textures
        let mut added_handles = Vec::new();
        for handle in user_textures.added_textures() {
            if let Some(texture_impl) = asset_mapping.get(&handle) {
                let egui_id = gui.add_texture(texture_impl.id());
                user_textures.register_texture(handle, egui_id);
                added_handles.push(handle);
            }
        }
        for handle in added_handles {
            user_textures.flush_added_texture(&handle);
        }
    }

    {
        // Sync Changed Textures
        let mut changed_handles = Vec::new();
        for handle in user_textures.changed_textures() {
            if let Some(texture_impl) = asset_mapping.get(&handle) {
                let egui_id = user_textures.texture_id(&handle).unwrap();
                gui.replace_texture(egui_id, texture_impl.id());
                changed_handles.push(handle);
            }
        }
        for handle in changed_handles {
            user_textures.flush_changed_texture(&handle);
        }
    }

    {
        // Sync Removed Textures
        let mut removed_handles = Vec::new();
        for handle in user_textures.removed_textures() {
            let egui_id = user_textures.deregister_texture(&handle);
            gui.remove_texture(egui_id);
            removed_handles.push(handle);
        }
        for handle in removed_handles {
            user_textures.flush_removed_texture(&handle);
        }
    }
}
