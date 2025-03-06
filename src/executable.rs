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
    pub assets: Option<PathBuf>,
}

impl Executable {
    pub fn assets(&self) -> Option<PathBuf> {
        self.assets.as_ref().map(|d| if d.is_relative() {
            self.bin.parent().map(|p|p.join(d)).expect("Missing executable path (is relative or root)")
        } else {
            d.to_path_buf()
        })
    }
}
