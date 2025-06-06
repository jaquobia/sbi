use std::path::Path;

use serde::{Deserialize, Serialize};
use anyhow::Result;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModManifestMod {
    pub publishedfileid: String,
    /// A Timestamp of when the currently installed artifact was uploaded to steam
    pub version: u64
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ModManifestJson {
    pub mods: Vec<ModManifestMod>
}

pub async fn read_manifest(folder: &Path) -> Result<ModManifestJson> {
    let manifest_path = folder.join("manifest.json");
    let manifest_bytes = tokio::fs::read(manifest_path).await?;
    let manifest: ModManifestJson = serde_json::from_slice(&manifest_bytes)?;
    Ok(manifest)
}

pub async fn write_manifest(folder: &Path, manifest: &ModManifestJson) -> Result<()> {
    let manifest_path = folder.join("manifest.json");
    let manifest_bytes = serde_json::to_vec(manifest)?;
    tokio::fs::write(manifest_path, manifest_bytes).await?;
    Ok(())
}
