use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// All executables should be some variant of these:  
/// - XStarbound - will enable the removal of automatic UGC loading through the ```-noworkshop``` flag  
/// - OpenStarbound - will enable the removal of automatic UGC loading through the ```"includeUGC": false``` field in sbinit.config
/// - Vanilla - has no current method for disabling UGC content
pub enum ExecutableVariant {
    XStarbound,
    OpenStarbound,
    Vanilla
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Executable {
    /// Path to the starbound binary
    pub bin: PathBuf,
    /// Assets to load in addition to vanilla assets
    /// For the vanilla executable, if this is None, there will be NO safe-guard to ensure vanilla
    /// assets are loaded, please ensure the vanilla executable has this field listed.
    pub assets: Option<PathBuf>,
}
