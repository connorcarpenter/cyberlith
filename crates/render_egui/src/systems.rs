use bevy_ecs::{
    change_detection::DetectChanges,
    system::{NonSendMut, Res, ResMut},
    world::World,
};

use render_api::base::Texture2D;
use render_glow::{core::Texture2DImpl, window::FrameInput, AssetImpls};

use crate::{EguiContext, EguiUserTextures, GUI};

pub fn startup(world: &mut World) {
    world.insert_non_send_resource(GUI::default());
}

pub fn pre_update(
    mut frame_input: NonSendMut<FrameInput<()>>,
    mut gui: NonSendMut<GUI>,
    egui_context: Res<EguiContext>,
) {
    gui.pre_update(egui_context.inner(), frame_input.as_mut());
}

pub fn post_update(
    mut frame_input: NonSendMut<FrameInput<()>>,
    mut gui: NonSendMut<GUI>,
    egui_context: Res<EguiContext>,
) {
    gui.post_update(egui_context.inner(), &mut frame_input.events);
}

pub fn draw(
    mut frame_input: NonSendMut<FrameInput<()>>,
    mut gui: NonSendMut<GUI>,
    egui_context: Res<EguiContext>,
) {
    gui.render(egui_context.inner());
}

pub fn sync(
    mut gui: NonSendMut<GUI>,
    mut user_textures: ResMut<EguiUserTextures>,
    mut texture_impls: ResMut<AssetImpls<Texture2D, Texture2DImpl>>,
) {
    if !user_textures.is_changed() {
        return;
    }

    // Sync Added Textures
    let added_handles = user_textures.flush_added_textures();
    for handle in added_handles {
        let texture_impl = texture_impls.get(&handle).unwrap();
        let egui_id = gui.add_texture(texture_impl.id());
        user_textures.register_texture(handle, egui_id);
    }

    // Sync Removed Textures
    let removed_handles = user_textures.flush_removed_textures();
    for handle in removed_handles {
        let egui_id = user_textures.deregister_texture(&handle);
        gui.remove_texture(egui_id);
    }
}
