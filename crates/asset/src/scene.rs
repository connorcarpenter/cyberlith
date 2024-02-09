use bevy_log::info;

use asset_io::FileTransformEntityType;
use math::{Quat, Vec3};
use render_api::components::Transform;
use storage::{AssetHash, Handle};

use crate::{
    asset_dependency::{AssetDependency, SkinOrScene, SkinOrSceneHandle},
    asset_handle::AssetHandleImpl,
    AssetHandle, SkinData,
};

impl AssetHash<SceneData> for String {}

pub struct SceneData {
    skin_or_scene_files: Vec<SkinOrScene>,
    net_transforms: Vec<(usize, Transform)>,
    computed_components: Option<Vec<(SkinOrSceneHandle, Transform)>>,
}

impl Default for SceneData {
    fn default() -> Self {
        panic!("");
    }
}

impl SceneData {
    pub(crate) fn load_dependencies(
        &self,
        handle: Handle<Self>,
        dependencies: &mut Vec<(AssetHandle, String)>,
    ) {
        for file in self.skin_or_scene_files.iter() {
            match file {
                SkinOrScene::Skin(AssetDependency::<SkinData>::Path(path)) => {
                    dependencies.push((handle.into(), path.clone()));
                }
                SkinOrScene::Scene(AssetDependency::<SceneData>::Path(path)) => {
                    dependencies.push((handle.into(), path.clone()));
                }
                _ => {
                    panic!("expected unloaded (no handles!) skin or scene file");
                }
            }
        }
    }

    pub(crate) fn finish_dependency(
        &mut self,
        dependency_path: String,
        dependency_handle: AssetHandle,
    ) {
        match dependency_handle.to_impl() {
            AssetHandleImpl::Skin(handle) => {
                info!(
                    "finished scene dependency for skin: {}, path: {}",
                    &handle.id, &dependency_path
                );
                let handle = SkinOrSceneHandle::Skin(handle);
                finish_skin_or_scene_dependency(
                    &mut self.skin_or_scene_files,
                    dependency_path,
                    handle,
                );
            }
            AssetHandleImpl::Scene(handle) => {
                let handle = SkinOrSceneHandle::Scene(handle);
                finish_skin_or_scene_dependency(
                    &mut self.skin_or_scene_files,
                    dependency_path,
                    handle,
                );
            }
            _ => {
                panic!("unexpected type of handle");
            }
        }

        if self.all_dependencies_loaded() {
            // compute components
            let mut components = Vec::new();
            for (file_index, transform) in self.net_transforms.iter() {
                let file = &self.skin_or_scene_files[*file_index];
                match file {
                    SkinOrScene::Skin(AssetDependency::<SkinData>::Handle(handle)) => {
                        components.push((SkinOrSceneHandle::Skin(*handle), *transform));
                    }
                    SkinOrScene::Scene(AssetDependency::<SceneData>::Handle(handle)) => {
                        components.push((SkinOrSceneHandle::Scene(*handle), *transform));
                    }
                    _ => panic!("impossible"),
                }
            }
            self.computed_components = Some(components);
        }
    }

    fn all_dependencies_loaded(&self) -> bool {
        for file in self.skin_or_scene_files.iter() {
            match file {
                SkinOrScene::Skin(AssetDependency::<SkinData>::Handle(_)) => {}
                SkinOrScene::Scene(AssetDependency::<SceneData>::Handle(_)) => {}
                _ => {
                    return false;
                }
            }
        }
        true
    }

    pub(crate) fn get_components(&self) -> Option<&Vec<(SkinOrSceneHandle, Transform)>> {
        self.computed_components.as_ref()
    }
}

pub(crate) fn finish_skin_or_scene_dependency(
    skin_or_scene_files: &mut Vec<SkinOrScene>,
    dependency_path: String,
    handle: SkinOrSceneHandle,
) {
    let mut found = false;
    for file in skin_or_scene_files.iter_mut() {
        match file {
            SkinOrScene::Skin(AssetDependency::<SkinData>::Path(path)) => {
                if path == &dependency_path {
                    let SkinOrSceneHandle::Skin(handle) = handle else {
                        panic!("expected skin handle");
                    };
                    *file = SkinOrScene::Skin(AssetDependency::<SkinData>::Handle(handle));
                    found = true;
                    break;
                }
            }
            SkinOrScene::Scene(AssetDependency::<SceneData>::Path(path)) => {
                if path == &dependency_path {
                    let SkinOrSceneHandle::Scene(handle) = handle else {
                        panic!("expected scene handle");
                    };
                    *file = SkinOrScene::Scene(AssetDependency::<SceneData>::Handle(handle));
                    found = true;
                    break;
                }
            }
            _ => {}
        }
    }
    if !found {
        panic!("unable to find dependency path for: {:?}", &dependency_path);
    }
}

impl From<String> for SceneData {
    fn from(path: String) -> Self {
        let file_path = format!("assets/{}", path);

        let Ok(data) = web_fs::read(&file_path) else {
            panic!("unable to read file: {:?}", &file_path);
        };

        let actions = asset_io::SceneAction::read(&data).expect("unable to parse file");

        info!("--- reading scene: {} ---", path);

        let mut skin_or_scene_files = Vec::new();
        let mut net_transforms = Vec::new();
        let mut file_index = 0;
        for action in actions {
            match action {
                asset_io::SceneAction::SkinOrSceneFile(path, name, file_type) => {
                    info!(
                        "SkinOrSceneFile {} - type: {:?}, path: {}/{}. ",
                        file_index, file_type, path, name
                    );

                    let asset_dependency =
                        match file_type {
                            FileTransformEntityType::Skin => {
                                SkinOrScene::Skin(AssetDependency::<SkinData>::Path(format!(
                                    "{}/{}",
                                    path, name
                                )))
                            }
                            FileTransformEntityType::Scene => SkinOrScene::Scene(
                                AssetDependency::<SceneData>::Path(format!("{}/{}", path, name)),
                            ),
                        };

                    skin_or_scene_files.push(asset_dependency);

                    file_index += 1;
                }
                asset_io::SceneAction::NetTransform(
                    file_index,
                    x,
                    y,
                    z,
                    scale_x,
                    scale_y,
                    scale_z,
                    rotation,
                ) => {
                    info!("NetTransform {} : file_index: {}, position ({} {} {}), scale: ({} {} {}), rotation: ({}, {}, {}, {})",
                             net_transforms.len(),
                             file_index,
                             x, y, z,
                             scale_x, scale_y, scale_z,
                             rotation.x, rotation.y, rotation.z, rotation.w);
                    let transform =
                        Transform::from_translation(Vec3::new(x as f32, y as f32, z as f32))
                            .with_scale(Vec3::new(scale_x, scale_y, scale_z))
                            .with_rotation(Quat::from_xyzw(
                                rotation.x, rotation.y, rotation.z, rotation.w,
                            ));
                    net_transforms.push((file_index as usize, transform));
                }
            }
        }

        info!("--- done reading scene ---");

        Self {
            skin_or_scene_files,
            net_transforms,
            computed_components: None,
        }
    }
}
