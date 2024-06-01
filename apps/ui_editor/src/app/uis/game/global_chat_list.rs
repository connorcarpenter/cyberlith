use game_engine::{
    asset::{AssetId, ETag},
};
use ui_builder::{Alignment, UiConfig, UiConfigBuild};

#[allow(unused)]
pub fn ui_define() -> (String, AssetId, ETag, UiConfig) {
    // config
    let ui_name = "global_chat_list";
    let ui_asset_id_str = "ws8m4d"; //AssetId::gen_random().as_string(); // keep this around to generate new AssetIds if needed!
    let text_icon_asset_id_str = "34mvvk"; // this probably shouldn't change, it's the text font
    let eye_icon_asset_id_str = "qbgz5j"; // this probably shouldn't change, it's the password eye
    let ui_etag = ETag::gen_random();

    // asset ids ..
    let ui_asset_id = AssetId::from_str(&ui_asset_id_str).unwrap();
    let text_icon_asset_id = AssetId::from_str(text_icon_asset_id_str).unwrap();
    let eye_icon_asset_id = AssetId::from_str(eye_icon_asset_id_str).unwrap();

    // Create UI !
    let mut ui_config = UiConfig::new();

    // styles
    let container_style = ui_config.create_panel_style(|s| {
        s.set_background_alpha(0.)
            .set_vertical()
            .set_children_valign(Alignment::End)
            .set_children_halign(Alignment::Start)
            .set_width_pc(100.0)
            .set_height_pc(100.0);
    });

    // nodes
    ui_config
        .set_text_icon_asset_id(&text_icon_asset_id)
        .set_eye_icon_asset_id(&eye_icon_asset_id)
        .root_mut()
        .set_style(container_style);

    (ui_name.to_string(), ui_asset_id, ui_etag, ui_config)
}
