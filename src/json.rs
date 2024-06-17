use std::path::PathBuf;

use rustc_hash::FxHashMap as HashMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct InstanceDataJson {
    pub name: String,
    pub executable: Option<String>,
    pub additional_assets: Option<Vec<String>>,
    pub collection_id: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct SBIConfigExecutable {
    pub bin: String,
    pub ld_path: Option<String>,
    pub custom_assets: Option<String>,
}

#[derive(Serialize, Deserialize, Default)]
pub struct SBIConfig {
    pub executables: HashMap<String, SBIConfigExecutable>,
}

#[derive(Serialize, Deserialize)]
pub struct SBILaunchMessageJson {
    pub exececutable_path: PathBuf,
    pub instance_path: Option<PathBuf>,
    pub ld_library_path: Option<PathBuf>,
}
