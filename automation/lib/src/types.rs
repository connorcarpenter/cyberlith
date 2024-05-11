
// TargetEnv
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum TargetEnv {
    Local,
    Prod,
}

impl TargetEnv {
    pub fn to_string(&self) -> String {
        match self {
            TargetEnv::Local => "local".to_string(),
            TargetEnv::Prod => "prod".to_string(),
        }
    }

    pub fn cargo_env(&self) -> String {
        match self {
            TargetEnv::Local => "debug".to_string(),
            TargetEnv::Prod => "release".to_string(),
        }
    }

    pub fn feature_flag(&self) -> String {
        match self {
            TargetEnv::Local => "local".to_string(),
            TargetEnv::Prod => "prod".to_string(),
        }
    }
}

// Output Type
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum OutputType {
    Bits, Json,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum FileExtension {
    Wasm,
    Js,
    Html,
}

impl FileExtension {
    pub fn to_string(&self) -> String {
        match self {
            FileExtension::Wasm => "wasm".to_string(),
            FileExtension::Js => "js".to_string(),
            FileExtension::Html => "html".to_string(),
        }
    }
}