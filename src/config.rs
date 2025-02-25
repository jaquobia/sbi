use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{executable::Executable, SBI_CONFIG_JSON_NAME};

fn default_executable() -> String { String::from("vanilla") }

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct SBIConfig {
    pub executables: rustc_hash::FxHashMap<String, Executable>,
    #[serde(default = "default_executable")]
    pub default_executable: String,
}

/// Load config from disk at `dir/config.json`
/// On error, returns default value for [SBIConfig](crate::config::SBIConfig)
pub async fn load_config(dir: PathBuf) -> SBIConfig {
    match load_config_failable(dir).await {
        Ok(config) => config,
        Err(e) => {
            log::error!("{e}");
            SBIConfig::default()
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
