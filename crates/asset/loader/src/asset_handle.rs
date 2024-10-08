use std::hash::{Hash, Hasher};

use bevy_ecs::component::Component;

use asset_id::{AssetId, AssetType};

use crate::{
    AnimatedModelData, AnimationData, IconData, MeshData, ModelData, MovementConfigData,
    PaletteData, SceneData, SkeletonData, SkinData, UiDependencies, UnitData,
};

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
pub enum TypedAssetId {
    Mesh(AssetId),
    Skeleton(AssetId),
    Palette(AssetId),
    Animation(AssetId),
    Icon(AssetId),
    Skin(AssetId),
    Model(AssetId),
    Scene(AssetId),
    Ui(AssetId),
    AnimatedModel(AssetId),
    MovementConfig(AssetId),
    Unit(AssetId),
}

impl TypedAssetId {
    pub fn new(asset_id: AssetId, asset_type: AssetType) -> Self {
        match asset_type {
            AssetType::Mesh => Self::Mesh(asset_id),
            AssetType::Skeleton => Self::Skeleton(asset_id),
            AssetType::Palette => Self::Palette(asset_id),
            AssetType::Animation => Self::Animation(asset_id),
            AssetType::Icon => Self::Icon(asset_id),
            AssetType::Skin => Self::Skin(asset_id),
            AssetType::Model => Self::Model(asset_id),
            AssetType::Scene => Self::Scene(asset_id),
            AssetType::Ui => Self::Ui(asset_id),
            AssetType::AnimatedModel => Self::AnimatedModel(asset_id),
            AssetType::MovementConfig => Self::MovementConfig(asset_id),
            AssetType::Unit => Self::Unit(asset_id),
        }
    }

    pub fn get_id(&self) -> AssetId {
        match self {
            Self::Mesh(id) => *id,
            Self::Skeleton(id) => *id,
            Self::Palette(id) => *id,
            Self::Animation(id) => *id,
            Self::Icon(id) => *id,
            Self::Skin(id) => *id,
            Self::Model(id) => *id,
            Self::Scene(id) => *id,
            Self::Ui(id) => *id,
            Self::AnimatedModel(id) => *id,
            Self::MovementConfig(id) => *id,
            Self::Unit(id) => *id,
        }
    }

    pub fn get_type(&self) -> AssetType {
        match self {
            Self::Mesh(_) => AssetType::Mesh,
            Self::Skeleton(_) => AssetType::Skeleton,
            Self::Palette(_) => AssetType::Palette,
            Self::Animation(_) => AssetType::Animation,
            Self::Icon(_) => AssetType::Icon,
            Self::Skin(_) => AssetType::Skin,
            Self::Model(_) => AssetType::Model,
            Self::Scene(_) => AssetType::Scene,
            Self::Ui(_) => AssetType::Ui,
            Self::AnimatedModel(_) => AssetType::AnimatedModel,
            Self::MovementConfig(_) => AssetType::MovementConfig,
            Self::Unit(_) => AssetType::Unit,
        }
    }
}

#[derive(Debug, Component)]
pub struct AssetHandle<T> {
    asset_id: AssetId,
    phantom_t: std::marker::PhantomData<T>,
}

impl<T> AssetHandle<T> {
    pub fn new(asset_id: AssetId) -> Self {
        Self {
            asset_id,
            phantom_t: std::marker::PhantomData,
        }
    }

    pub fn asset_id(&self) -> AssetId {
        self.asset_id
    }
}

impl<T> Hash for AssetHandle<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.asset_id.hash(state);
    }
}

impl<T> PartialEq<Self> for AssetHandle<T> {
    fn eq(&self, other: &Self) -> bool {
        self.asset_id == other.asset_id
    }
}

impl<T> Eq for AssetHandle<T> {}

impl<T> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            asset_id: self.asset_id,
            phantom_t: std::marker::PhantomData,
        }
    }
}

impl<T> Copy for AssetHandle<T> {}

// TypedAssetId -> AssetHandle
impl From<TypedAssetId> for AssetHandle<SkeletonData> {
    fn from(typed_asset_id: TypedAssetId) -> Self {
        let TypedAssetId::Skeleton(asset_id) = typed_asset_id else {
            panic!("expected skeleton id");
        };
        Self::new(asset_id)
    }
}

impl From<TypedAssetId> for AssetHandle<MeshData> {
    fn from(typed_asset_id: TypedAssetId) -> Self {
        let TypedAssetId::Mesh(asset_id) = typed_asset_id else {
            panic!("expected mesh id");
        };
        Self::new(asset_id)
    }
}

impl From<TypedAssetId> for AssetHandle<AnimationData> {
    fn from(typed_asset_id: TypedAssetId) -> Self {
        let TypedAssetId::Animation(asset_id) = typed_asset_id else {
            panic!("expected animation id");
        };
        Self::new(asset_id)
    }
}

impl From<TypedAssetId> for AssetHandle<SkinData> {
    fn from(typed_asset_id: TypedAssetId) -> Self {
        let TypedAssetId::Skin(asset_id) = typed_asset_id else {
            panic!("expected skin id");
        };
        Self::new(asset_id)
    }
}

impl From<TypedAssetId> for AssetHandle<PaletteData> {
    fn from(typed_asset_id: TypedAssetId) -> Self {
        let TypedAssetId::Palette(asset_id) = typed_asset_id else {
            panic!("expected palette id");
        };
        Self::new(asset_id)
    }
}

impl From<TypedAssetId> for AssetHandle<SceneData> {
    fn from(typed_asset_id: TypedAssetId) -> Self {
        let TypedAssetId::Scene(asset_id) = typed_asset_id else {
            panic!("expected scene id");
        };
        Self::new(asset_id)
    }
}

impl From<TypedAssetId> for AssetHandle<ModelData> {
    fn from(typed_asset_id: TypedAssetId) -> Self {
        let TypedAssetId::Model(asset_id) = typed_asset_id else {
            panic!("expected model id");
        };
        Self::new(asset_id)
    }
}

impl From<TypedAssetId> for AssetHandle<IconData> {
    fn from(typed_asset_id: TypedAssetId) -> Self {
        let TypedAssetId::Icon(asset_id) = typed_asset_id else {
            panic!("expected icon id");
        };
        Self::new(asset_id)
    }
}

impl From<TypedAssetId> for AssetHandle<UiDependencies> {
    fn from(typed_asset_id: TypedAssetId) -> Self {
        let TypedAssetId::Ui(asset_id) = typed_asset_id else {
            panic!("expected ui id");
        };
        Self::new(asset_id)
    }
}

impl From<TypedAssetId> for AssetHandle<AnimatedModelData> {
    fn from(typed_asset_id: TypedAssetId) -> Self {
        let TypedAssetId::AnimatedModel(asset_id) = typed_asset_id else {
            panic!("expected animated model id");
        };
        Self::new(asset_id)
    }
}

impl From<TypedAssetId> for AssetHandle<MovementConfigData> {
    fn from(typed_asset_id: TypedAssetId) -> Self {
        let TypedAssetId::MovementConfig(asset_id) = typed_asset_id else {
            panic!("expected movement config id");
        };
        Self::new(asset_id)
    }
}

impl From<TypedAssetId> for AssetHandle<UnitData> {
    fn from(typed_asset_id: TypedAssetId) -> Self {
        let TypedAssetId::Unit(asset_id) = typed_asset_id else {
            panic!("expected unit id");
        };
        Self::new(asset_id)
    }
}

// AssetHandle -> TypedAssetId

impl From<AssetHandle<SkeletonData>> for TypedAssetId {
    fn from(handle: AssetHandle<SkeletonData>) -> Self {
        Self::new(handle.asset_id, AssetType::Skeleton)
    }
}

impl From<AssetHandle<MeshData>> for TypedAssetId {
    fn from(handle: AssetHandle<MeshData>) -> Self {
        Self::new(handle.asset_id, AssetType::Mesh)
    }
}

impl From<AssetHandle<AnimationData>> for TypedAssetId {
    fn from(handle: AssetHandle<AnimationData>) -> Self {
        Self::new(handle.asset_id, AssetType::Animation)
    }
}

impl From<AssetHandle<SkinData>> for TypedAssetId {
    fn from(handle: AssetHandle<SkinData>) -> Self {
        Self::new(handle.asset_id, AssetType::Skin)
    }
}

impl From<AssetHandle<PaletteData>> for TypedAssetId {
    fn from(handle: AssetHandle<PaletteData>) -> Self {
        Self::new(handle.asset_id, AssetType::Palette)
    }
}

impl From<AssetHandle<SceneData>> for TypedAssetId {
    fn from(handle: AssetHandle<SceneData>) -> Self {
        Self::new(handle.asset_id, AssetType::Scene)
    }
}

impl From<AssetHandle<ModelData>> for TypedAssetId {
    fn from(handle: AssetHandle<ModelData>) -> Self {
        Self::new(handle.asset_id, AssetType::Model)
    }
}

impl From<AssetHandle<IconData>> for TypedAssetId {
    fn from(handle: AssetHandle<IconData>) -> Self {
        Self::new(handle.asset_id, AssetType::Icon)
    }
}

impl From<AssetHandle<UiDependencies>> for TypedAssetId {
    fn from(handle: AssetHandle<UiDependencies>) -> Self {
        Self::new(handle.asset_id, AssetType::Ui)
    }
}

impl From<AssetHandle<AnimatedModelData>> for TypedAssetId {
    fn from(handle: AssetHandle<AnimatedModelData>) -> Self {
        Self::new(handle.asset_id, AssetType::AnimatedModel)
    }
}

impl From<AssetHandle<MovementConfigData>> for TypedAssetId {
    fn from(handle: AssetHandle<MovementConfigData>) -> Self {
        Self::new(handle.asset_id, AssetType::MovementConfig)
    }
}

impl From<AssetHandle<UnitData>> for TypedAssetId {
    fn from(handle: AssetHandle<UnitData>) -> Self {
        Self::new(handle.asset_id, AssetType::Unit)
    }
}
