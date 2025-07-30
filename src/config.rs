use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{executable::Executable, SBI_CONFIG_JSON_NAME};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SBIConfig {
    pub executables: rustc_hash::FxHashMap<String, Executable>,
    pub default_executable: Option<String>,
    #[serde(default)]
    pub close_on_launch: bool,
}

/// Load config from disk at `dir/config.json`
/// On error, returns default value for [SBIConfig](crate::config::SBIConfig)
pub async fn load_config(dir: PathBuf) -> SBIConfig {
    match load_config_failable(dir.clone()).await {
        Ok(config) => config,
        Err(e) => {
            // TODO: Handle write error.
            log::warn!("Error reading config file: {e}. Writing default (empty) values.");
            log::info!("Ignore this line, the logger deletes a potential error if this line does not exist due to a deduplication bug.");
            let config = SBIConfig::default();
            let _res = write_config_to_disk(dir, config.clone()).await;
            config
        }
    }
}

/// Load config from disk at `dir/config.json`
async fn load_config_failable(dir: PathBuf) -> anyhow::Result<SBIConfig> {
    let bytes = tokio::fs::read(dir.join(SBI_CONFIG_JSON_NAME)).await?;
    let config = serde_json::from_slice::<SBIConfig>(&bytes)?;
    Ok(config)
}

/// Write [config](crate::config::SBIConfig) to disk at `dir/config.json`
pub async fn write_config_to_disk(dir: PathBuf, config: SBIConfig) -> anyhow::Result<()> {
    let bytes = serde_json::to_vec(&config)?;
    tokio::fs::write(dir.join(SBI_CONFIG_JSON_NAME), &bytes).await?;
    Ok(())
}
