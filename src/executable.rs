use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// All executables should be some variant of these:  
/// - XStarbound - will enable the removal of automatic UGC loading through the ```-noworkshop``` flag  
/// - OpenStarbound - will enable the removal of automatic UGC loading through the ```"includeUGC": false``` field in sbinit.config
/// - Vanilla - has no current method for disabling UGC content
//TODO: Impement onto executables, this is currently unused
#[derive(Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Debug, Clone)]
pub enum ExecutableVariant {
    XStarbound,
    OpenStarbound,
    Vanilla,
}

impl ExecutableVariant {
    pub fn options() -> [Self; 3] {
        [Self::XStarbound, Self::OpenStarbound, Self::Vanilla]
    }
}

/// TODO: Decide whether to keep this or not.
/// Only needed to handle edge-case of config missing a value but being necessary
impl Default for ExecutableVariant {
    fn default() -> Self {
        Self::Vanilla
    }
}

impl std::fmt::Display for ExecutableVariant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::XStarbound => "XStarbound",
            Self::OpenStarbound => "OpenStarbound",
            Self::Vanilla => "Vanilla",
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Executable {
    /// Path to the starbound binary
    pub bin: PathBuf,
    /// Assets to load in addition to vanilla assets
    pub assets: Option<PathBuf>,
    #[serde(default)]
    pub variant: ExecutableVariant,
}

impl Executable {
    pub fn assets(&self) -> Option<PathBuf> {
        self.assets.as_ref().map(|d| {
            if d.is_relative() {
                self.bin
                    .parent()
                    .map(|p| p.join(d))
                    .expect("Missing executable path (is relative or root)")
            } else {
                d.to_path_buf()
            }
        })
    }
}
